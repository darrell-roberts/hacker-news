use crate::{
    api::Story, SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_CATEGORY,
    ITEM_DESCENDANT_COUNT, ITEM_ID, ITEM_KIDS, ITEM_PARENT_ID, ITEM_RANK, ITEM_SCORE,
    ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE, ITEM_TYPE, ITEM_URL,
};
use futures_core::Stream;
use futures_util::{pin_mut, stream::FuturesUnordered, StreamExt, TryFutureExt, TryStreamExt};
use hacker_news_api::{ApiClient, ArticleType, Item};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    convert::identity,
    mem,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tantivy::{schema::Field, IndexWriter, TantivyDocument, Term};
use tokio::{
    sync::mpsc::{self, UnboundedSender},
    time::timeout,
};
#[cfg(feature = "trace")]
use tracing::instrument;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_documents: u64,
    pub total_comments: u64,
    pub total_stories: u64,
    pub total_jobs: u64,
    pub total_polls: u64,
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

struct Fields {
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

impl Fields {
    fn new(ctx: &SearchContext) -> Result<Self, SearchError> {
        Ok(Self {
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
}

struct WriteContext<'a> {
    writer: IndexWriter,
    story_category: &'a str,

    fields: Fields,
}

impl<'a> WriteContext<'a> {
    fn new(
        fields: Fields,
        writer: IndexWriter,
        story_category: &'a str,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            writer,
            story_category,
            fields,
        })
    }

    fn write_story(&self, item: StoryRef) -> Result<(), SearchError> {
        let StoryRef { story: item, rank } = item;
        self.write_doc(&item, rank, None)
    }

    fn write_comment(&self, comment: CommentRef) -> Result<(), SearchError> {
        let CommentRef {
            story_id,
            comment,
            rank,
        } = comment;
        self.write_doc(&comment, rank, Some(story_id))
            .inspect_err(|err| {
                error!("Failed to write doc: {err}");
            })
    }

    fn write_doc(&self, item: &Item, rank: u64, story_id: Option<u64>) -> Result<(), SearchError> {
        let mut doc = TantivyDocument::new();

        doc.add_u64(self.fields.rank, rank);
        doc.add_u64(self.fields.id, item.id);
        if let Some(id) = item.parent {
            doc.add_u64(self.fields.parent, id);
        }
        if let Some(t) = item.title.as_deref() {
            doc.add_text(self.fields.title, t);
        }
        if let Some(t) = item.text.as_deref() {
            doc.add_text(self.fields.body, t);
        }
        if let Some(u) = item.url.as_deref() {
            doc.add_text(self.fields.url, u);
        }
        doc.add_text(self.fields.by, &item.by);
        doc.add_text(self.fields.ty, &item.ty);

        if let Some(n) = item.descendants {
            doc.add_u64(self.fields.descendant_count, n);
        }

        if let Some(id) = story_id {
            doc.add_u64(self.fields.parent_story, id);
        }

        if item.ty == "story" {
            doc.add_text(self.fields.category, self.story_category);
            doc.add_u64(self.fields.score, item.score);
        }

        doc.add_u64(self.fields.time, item.time);

        for id in &item.kids {
            doc.add_u64(self.fields.kids, *id);
        }

        self.writer.add_document(doc)?;
        Ok(())
    }

    /// Delete a story and all it's child comments.
    fn delete_story(&self, story: &Story) {
        self.writer
            .delete_term(Term::from_field_u64(self.fields.id, story.id));
        self.writer
            .delete_term(Term::from_field_u64(self.fields.parent_story, story.id));
    }

    /// Delete all documents from the active index.
    fn delete_all_docs(&mut self) -> Result<u64, SearchError> {
        Ok(self.writer.delete_all_documents()?)
    }

    /// Commit changes to the index.
    fn commit(&mut self) -> Result<u64, SearchError> {
        Ok(self.writer.commit()?)
    }
}

/// Yield a stream of comments for the given comment_ids.
#[cfg_attr(feature = "trace", instrument(skip_all))]
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
        .inspect_err(move |err| {
            error!("Failed to fetch comments for story_id {story_id}: {err}");
        })
}

/// Recurse through all child comments and send each one to the index
/// writer channel.
#[cfg_attr(feature = "trace", instrument(skip_all))]
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
        if tx.is_closed() {
            error!("index writer channel is closed");
            break;
        }

        if let Err(err) = tx.send(ItemRef::Comment(comment)) {
            error!("Failed to send comment {err}");
        }
    }

    Ok(())
}

/// Get the story and nested comments from the firebase REST api and send each
/// document to the index writer channel.
#[cfg_attr(feature = "trace", instrument(skip_all, fields(story_id = story.id)))]
async fn collect_story(
    client: Arc<ApiClient>,
    tx: UnboundedSender<ItemRef>,
    mut story: Item,
    rank: u64,
) -> Result<(), SearchError> {
    let story_id = story.id;
    info!("Collecting comments for story_id {story_id}");
    timeout(
        Duration::from_secs(60),
        send_comments(&client, story.id, mem::take(&mut story.kids), tx.clone()),
    )
    .await
    .map_err(|_| SearchError::TimedOut(format!("story_id {story_id}, sending comments")))??;

    if tx.is_closed() {
        error!("index writer channel is closed");
        return Ok(());
    }

    if let Err(err) = tx.send(ItemRef::Story(StoryRef { story, rank })) {
        error!("Failed to send story {err}");
    }
    Ok(())
}

/// Get all stories and nested comments for the given category and send
/// each document to the index writer channel.
#[cfg_attr(feature = "trace", instrument(skip(tx)))]
async fn collect(
    tx: UnboundedSender<ItemRef>,
    category_type: ArticleType,
) -> Result<(), SearchError> {
    let client = Arc::new(ApiClient::new()?);
    let stories = client.articles(75, category_type).await?;

    info!("Building {} top docs for {category_type}", stories.len());

    let mut handles = stories
        .into_iter()
        .zip(1..)
        .map(|(story, rank)| {
            let client = client.clone();
            let tx = tx.clone();
            let story_id = story.id;
            timeout(
                Duration::from_secs(60 * 3),
                collect_story(client, tx, story, rank),
            )
            .map_err(move |_| SearchError::TimedOut(format!("collecting story: {story_id}")))
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(result) = handles.next().await {
        let r = result.and_then(identity);
        if let Err(err) = r {
            error!("Failed to join story task: {err}");
        }
    }

    Ok(())
}

#[cfg_attr(feature = "trace", instrument(skip(ctx)))]
pub async fn rebuild_index(
    ctx: Arc<RwLock<SearchContext>>,
    category_type: ArticleType,
) -> Result<IndexStats, SearchError> {
    let start_time = Instant::now();
    info!("Creating index for {category_type}");

    let mut writer_context = writer_context(ctx.clone(), category_type)?;
    writer_context.delete_all_docs()?;

    let (tx, mut rx) = mpsc::unbounded_channel::<ItemRef>();
    let result = tokio::spawn(collect(tx, category_type));

    while let Some(item) = rx.recv().await {
        match item {
            ItemRef::Story(s) => writer_context.write_story(s)?,
            ItemRef::Comment(c) => writer_context.write_comment(c)?,
        }
    }

    info!("Finished indexing");

    result.await??;
    writer_context.commit()?;

    let g = ctx.read().unwrap();
    if g.active_index == category_type {
        g.reader.reload()?;
    }
    document_stats(&g, start_time.elapsed(), category_type)
}

fn writer_context(
    ctx: Arc<RwLock<SearchContext>>,
    category_type: ArticleType,
) -> Result<WriteContext<'static>, SearchError> {
    let g = ctx.read().unwrap();
    let index = g.indices.get(category_type.as_str()).unwrap();
    let fields = Fields::new(&g)?;
    WriteContext::new(fields, index.writer(50_000_000)?, category_type.as_str())
}

pub async fn update_story(
    ctx: Arc<RwLock<SearchContext>>,
    story: Story,
    category_type: ArticleType,
) -> Result<(), SearchError> {
    let api = Arc::new(ApiClient::new()?);
    let latest = api.item(story.id).await?;
    let story_id = story.id;

    if latest.descendants.unwrap_or_default() != story.descendants {
        info!(
            "New comments {}.. re-indexing story {story_id}",
            latest.descendants.unwrap_or_default()
        );

        let writer_context = writer_context(ctx.clone(), category_type)?;

        rebuild_story(api, writer_context, story, latest).await?;

        let g = ctx.read().unwrap();
        if g.active_index == category_type {
            g.reader.reload()?;
        }
        info!("Rebuilt story {story_id}");
    }

    Ok(())
}

async fn rebuild_story(
    client: Arc<ApiClient>,
    mut writer_context: WriteContext<'_>,
    story: Story,
    latest: Item,
) -> Result<(), SearchError> {
    writer_context.delete_story(&story);
    let (tx, mut rx) = mpsc::unbounded_channel::<ItemRef>();

    let result = tokio::spawn(collect_story(client, tx, latest, story.rank));

    while let Some(item) = rx.recv().await {
        match item {
            ItemRef::Story(s) => writer_context.write_story(s)?,
            ItemRef::Comment(c) => writer_context.write_comment(c)?,
        }
    }
    result.await??;
    writer_context.commit()?;
    Ok(())
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
    let searcher = if ctx.active_index == category {
        ctx.searcher()
    } else {
        ctx.indices
            .get(category.as_str())
            .unwrap()
            .reader()?
            .searcher()
    };

    let type_field = ctx.schema.get_field(ITEM_TYPE)?;

    let total_comments = searcher.doc_freq(&Term::from_field_text(type_field, "comment"))?;
    let total_jobs = searcher.doc_freq(&Term::from_field_text(type_field, "job"))?;
    let total_stories = searcher.doc_freq(&Term::from_field_text(type_field, "story"))?;
    let total_polls = searcher.doc_freq(&Term::from_field_text(type_field, "poll"))?;
    let total_documents = searcher.num_docs();

    Ok(IndexStats {
        total_documents,
        total_comments,
        total_jobs,
        total_stories,
        total_polls,
        build_time,
        built_on: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        category,
    })
}
