use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_CATEGORY, ITEM_DESCENDANT_COUNT, ITEM_ID,
    ITEM_KIDS, ITEM_PARENT_ID, ITEM_RANK, ITEM_SCORE, ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE,
    ITEM_TYPE, ITEM_URL,
};
use futures_core::Stream;
use futures_util::{pin_mut, TryStreamExt};
use hacker_news_api::{ApiClient, ArticleType, Item};
use serde::{Deserialize, Serialize};
use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};
use tantivy::{schema::Field, IndexWriter, TantivyDocument, Term};
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    time::timeout,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: u64,
    pub total_comments: u64,
    pub build_time: Duration,
    pub built_on: u64,
    pub category: ArticleType,
}

struct CommentRef {
    story_id: u64,
    comment: Item,
    rank: u64,
}

struct StoryRef {
    story: Item,
    rank: u64,
}

enum ItemRef {
    Story(StoryRef),
    Comment(CommentRef),
}

struct WriteContext<'a> {
    writer: &'a mut IndexWriter,
    story_category: &'a str,

    id: Field,
    parent: Field,
    title: Field,
    body: Field,
    url: Field,
    by: Field,
    ty: Field,
    rank: Field,
    descendant_count: Field,
    category: Field,
    time: Field,
    parent_story: Field,
    kids: Field,
    score: Field,
}

impl<'a> WriteContext<'a> {
    fn new(
        ctx: &'a SearchContext,
        writer: &'a mut IndexWriter,
        story_category: &'a str,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            writer,
            story_category,

            id: ctx.schema.get_field(ITEM_ID)?,
            parent: ctx.schema.get_field(ITEM_PARENT_ID)?,
            title: ctx.schema.get_field(ITEM_TITLE)?,
            body: ctx.schema.get_field(ITEM_BODY)?,
            url: ctx.schema.get_field(ITEM_URL)?,
            by: ctx.schema.get_field(ITEM_BY)?,
            ty: ctx.schema.get_field(ITEM_TYPE)?,
            rank: ctx.schema.get_field(ITEM_RANK)?,
            descendant_count: ctx.schema.get_field(ITEM_DESCENDANT_COUNT)?,
            category: ctx.schema.get_field(ITEM_CATEGORY)?,
            time: ctx.schema.get_field(ITEM_TIME)?,
            parent_story: ctx.schema.get_field(ITEM_STORY_ID)?,
            kids: ctx.schema.get_field(ITEM_KIDS)?,
            score: ctx.schema.get_field(ITEM_SCORE)?,
        })
    }

    fn write_story(&self, item: StoryRef) -> Result<(), SearchError> {
        let StoryRef { story: item, rank } = item;
        self.write_doc(&item, rank, None)?;
        Ok(())
    }

    fn write_comment(&self, comment: CommentRef) -> Result<(), SearchError> {
        let CommentRef {
            story_id,
            comment,
            rank,
        } = comment;
        self.write_doc(&comment, rank, Some(story_id))?;
        Ok(())
    }

    fn write_doc(&self, item: &Item, rank: u64, story_id: Option<u64>) -> Result<(), SearchError> {
        let mut doc = TantivyDocument::new();
        doc.add_u64(self.rank, rank);
        doc.add_u64(self.id, item.id);
        if let Some(id) = item.parent {
            doc.add_u64(self.parent, id);
        }
        if let Some(t) = item.title.as_deref() {
            doc.add_text(self.title, t);
        }
        if let Some(t) = item.text.as_deref() {
            doc.add_text(self.body, t);
        }
        if let Some(u) = item.url.as_deref() {
            doc.add_text(self.url, u);
        }
        doc.add_text(self.by, &item.by);
        doc.add_text(self.ty, &item.ty);

        if let Some(n) = item.descendants {
            doc.add_u64(self.descendant_count, n);
        }

        if let Some(id) = story_id {
            doc.add_u64(self.parent_story, id);
        }

        if item.ty == "story" {
            doc.add_text(self.category, self.story_category);
            doc.add_u64(self.score, item.score);
        }

        doc.add_u64(self.time, item.time);

        for id in &item.kids {
            doc.add_u64(self.kids, *id);
        }

        self.writer.add_document(doc)?;
        Ok(())
    }
}

fn comments_iter(
    client: &ApiClient,
    story_id: u64,
    comment_ids: &[u64],
) -> impl Stream<Item = Result<CommentRef, anyhow::Error>> {
    client
        .items_stream(comment_ids)
        .map_ok(move |(rank, item)| CommentRef {
            story_id,
            comment: item,
            rank,
        })
}

async fn send_comments(
    client: &ApiClient,
    story_id: u64,
    comment_ids: Vec<u64>,
    tx: UnboundedSender<ItemRef>,
) -> Result<(), SearchError> {
    let mut comment_stack = comments_iter(client, story_id, &comment_ids)
        .try_collect::<Vec<_>>()
        .await?;

    while let Some(comment) = comment_stack.pop() {
        let stream = comments_iter(client, story_id, &comment.comment.kids);
        pin_mut!(stream);
        while let Some(child) = stream.try_next().await? {
            comment_stack.push(child);
        }
        tx.send(ItemRef::Comment(comment)).unwrap();
    }

    Ok(())
}

async fn collect_story(
    client: Arc<ApiClient>,
    tx: UnboundedSender<ItemRef>,
    mut story: Item,
    rank: u64,
) -> Result<(), SearchError> {
    let story_id = story.id;
    timeout(
        Duration::from_secs(20),
        send_comments(&client, story.id, mem::take(&mut story.kids), tx.clone()),
    )
    .await
    .map_err(|_| SearchError::TimedOut(format!("story_id {story_id}, sending comments")))??;

    tx.send(ItemRef::Story(StoryRef { story, rank })).unwrap();
    Ok(())
}

async fn collect(
    tx: UnboundedSender<ItemRef>,
    category_type: ArticleType,
) -> Result<(), SearchError> {
    let client = Arc::new(ApiClient::new()?);

    let stories = client.articles(75, category_type).await?;

    println!("{} {category_type}", stories.len());

    let mut handles = Vec::new();
    // Spawn a task for every story. Each task will recursively index
    // all the comments.
    for (story, rank) in stories.into_iter().zip(1..) {
        let client = client.clone();
        let tx = tx.clone();
        let story_id = story.id;

        handles.push(tokio::spawn(async move {
            timeout(
                Duration::from_secs(60 * 2),
                collect_story(client, tx, story, rank),
            )
            .await
            .map_err(|_| SearchError::TimedOut(format!("collecting story: {story_id}")))?
        }));
    }

    for handle in handles {
        handle.await.unwrap()?;
    }

    Ok(())
}

pub async fn rebuild_index(
    ctx: &SearchContext,
    category_type: ArticleType,
) -> Result<IndexStats, SearchError> {
    let start_time = Instant::now();
    let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;
    writer.delete_all_documents()?;

    println!("Creating index for {category_type}");

    let (tx, mut rx) = mpsc::unbounded_channel::<ItemRef>();
    let result = tokio::spawn(async move { collect(tx, category_type).await });

    let writer_context = WriteContext::new(ctx, &mut writer, "top")?;
    while let Some(item) = rx.recv().await {
        match item {
            ItemRef::Story(s) => writer_context.write_story(s)?,
            ItemRef::Comment(c) => writer_context.write_comment(c)?,
        }
    }

    result.await.unwrap()?;
    writer.commit()?;

    document_stats(ctx, start_time.elapsed(), category_type)
}

// pub fn index_articles<'a>(
//     ctx: &'a SearchContext,
//     client: &'a ApiClient,
//     writer: &'a mut IndexWriter,
//     items: &'a [Item],
//     category: &'a str,
//     mut story_id: Option<u64>,
// ) -> Pin<Box<impl Future<Output = Result<(), SearchError>> + use<'a>>> {
//     Box::pin(async move {
//         let id = ctx.schema.get_field(ITEM_ID)?;
//         let parent = ctx.schema.get_field(ITEM_PARENT_ID)?;
//         let title = ctx.schema.get_field(ITEM_TITLE)?;
//         let body = ctx.schema.get_field(ITEM_BODY)?;
//         let url = ctx.schema.get_field(ITEM_URL)?;
//         let by = ctx.schema.get_field(ITEM_BY)?;
//         let ty = ctx.schema.get_field(ITEM_TYPE)?;
//         let rank = ctx.schema.get_field(ITEM_RANK)?;
//         let descendant_count = ctx.schema.get_field(ITEM_DESCENDANT_COUNT)?;
//         let category_field = ctx.schema.get_field(ITEM_CATEGORY)?;
//         let time = ctx.schema.get_field(ITEM_TIME)?;
//         let parent_story_id = ctx.schema.get_field(ITEM_STORY_ID)?;
//         let kids = ctx.schema.get_field(ITEM_KIDS)?;
//         let score = ctx.schema.get_field(ITEM_SCORE)?;

//         for (item, index) in items.iter().zip(1..) {
//             let mut doc = TantivyDocument::new();
//             doc.add_u64(rank, index);
//             doc.add_u64(id, item.id);
//             if let Some(id) = item.parent {
//                 doc.add_u64(parent, id);
//             }
//             if let Some(t) = item.title.as_deref() {
//                 doc.add_text(title, t);
//             }
//             if let Some(t) = item.text.as_deref() {
//                 doc.add_text(body, t);
//             }
//             if let Some(u) = item.url.as_deref() {
//                 doc.add_text(url, u);
//             }
//             doc.add_text(by, &item.by);
//             doc.add_text(ty, &item.ty);

//             if let Some(n) = item.descendants {
//                 doc.add_u64(descendant_count, n);
//             }

//             if let Some(id) = story_id {
//                 doc.add_u64(parent_story_id, id);
//             }

//             if item.ty == "story" {
//                 story_id = Some(item.id);
//                 doc.add_text(category_field, category);
//                 doc.add_u64(score, item.score);
//             }

//             doc.add_u64(time, item.time);

//             for id in &item.kids {
//                 doc.add_u64(kids, *id);
//             }

//             if !item.kids.is_empty() {
//                 let children = client.items(&item.kids).await?;
//                 index_articles(ctx, client, writer, &children, category, story_id).await?;
//             }

//             // TODO: Add a depth field for comments. This will allow for
//             // easy indentation for viewing the comment tree.

//             writer.add_document(doc)?;
//         }
//         Ok(())
//     })
// }

// pub async fn rebuild_index(ctx: &SearchContext) -> Result<(u64, u64), SearchError> {
//     let client = ApiClient::new()?;

//     let articles = client
//         .articles(75, hacker_news_api::ArticleType::Top)
//         .await?;

//     let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;
//     writer.delete_all_documents()?;

//     index_articles(ctx, &client, &mut writer, &articles, "top", None).await?;
//     writer.commit()?;

//     ctx.reader.reload()?;

//     document_stats(ctx)
// }

pub fn document_stats(
    ctx: &SearchContext,
    build_time: Duration,
    category: ArticleType,
) -> Result<IndexStats, SearchError> {
    let searcher = ctx.searcher();
    let total_comments = searcher.doc_freq(&Term::from_field_text(
        ctx.schema.get_field(ITEM_TYPE)?,
        "comment",
    ))?;

    let total_documents = searcher.num_docs();
    Ok(IndexStats {
        total_documents,
        total_comments,
        build_time,
        built_on: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        category,
    })
}
