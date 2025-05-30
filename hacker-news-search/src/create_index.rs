//! Create index.
use crate::{
    api::{Comment, Story},
    HackerNewsFields, SearchContext, SearchError, SearchResult, ITEM_TYPE,
};
use futures::{channel::mpsc, SinkExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use futures_util::stream::FuturesUnordered;
use hacker_news_api::{ApiClient, ArticleType, Item, ItemEventData};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::{
    convert::identity,
    future::ready,
    mem,
    sync::{Arc, OnceLock, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tantivy::{IndexWriter, TantivyDocument, Term};
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::AbortHandle,
    time::timeout,
};
#[cfg(feature = "trace")]
use tracing::{instrument, Instrument as _};

/// Single api client for connection pooling re-use.
static API: OnceLock<Arc<ApiClient>> = OnceLock::new();

pub fn api_client() -> Arc<ApiClient> {
    let client =
        API.get_or_init(|| Arc::new(ApiClient::new().expect("Could not create API client")));
    client.clone()
}

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

pub struct WriteContext<'a> {
    writer: IndexWriter,
    story_category: &'a str,
    fields: HackerNewsFields,
}

impl<'a> WriteContext<'a> {
    pub fn new(
        fields: HackerNewsFields,
        writer: IndexWriter,
        story_category: &'a str,
    ) -> SearchResult<Self> {
        Ok(Self {
            writer,
            story_category,
            fields,
        })
    }

    fn write_story(&self, item: StoryRef) -> SearchResult<()> {
        let StoryRef { story: item, rank } = item;
        self.write_doc(&item, rank, None)
    }

    fn write_comment(&self, comment: CommentRef) -> SearchResult<()> {
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

    fn write_doc(&self, item: &Item, rank: u64, story_id: Option<u64>) -> SearchResult<()> {
        let mut doc = TantivyDocument::new();

        doc.add_u64(self.fields.rank, rank);
        doc.add_u64(self.fields.id, item.id);
        if let Some(id) = item.parent {
            doc.add_u64(self.fields.parent_id, id);
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
            doc.add_u64(self.fields.story_id, id);
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
            .delete_term(Term::from_field_u64(self.fields.story_id, story.id));
    }

    /// Delete all documents from the active index.
    fn delete_all_docs(&mut self) -> SearchResult<u64> {
        Ok(self.writer.delete_all_documents()?)
    }

    /// Commit changes to the index.
    fn commit(&mut self) -> SearchResult<u64> {
        let ts = self.writer.commit()?;
        Ok(ts)
    }
}
/// Yield a stream of comments for the given comment_ids.
#[cfg_attr(feature = "trace", instrument(skip_all))]
fn comment_stream(
    client: &ApiClient,
    story_id: u64,
    comment_ids: &[u64],
) -> impl Stream<Item = CommentRef> {
    client
        .items(comment_ids)
        .inspect_err(|err| {
            error!("Failed to fetch comment: {err}");
        })
        .filter_map(|item| ready(item.ok()))
        .enumerate()
        .map(move |(index, item)| CommentRef {
            story_id,
            comment: item,
            rank: index as u64,
        })
}

/// Recurse through all child comments and send each one to the index
/// writer channel.
#[cfg_attr(feature = "trace", instrument(skip_all))]
async fn send_comments(
    client: &ApiClient,
    story_id: u64,
    comment_ids: Vec<u64>,
    tx: Sender<ItemRef>,
) {
    let mut comment_stack = comment_stream(client, story_id, &comment_ids)
        .collect::<Vec<_>>()
        .await;

    while let Some(comment) = comment_stack.pop() {
        let children = comment_stream(client, story_id, &comment.comment.kids)
            .collect::<Vec<_>>()
            .await;
        comment_stack.extend(children);

        if tx.is_closed() {
            error!("Index writer channel is closed");
            break;
        }

        if let Err(err) = tx.send(ItemRef::Comment(comment)).await {
            error!("Failed to send comment {err}");
        }
    }
}

/// Get the story and nested comments from the firebase REST api and send each
/// document to the index writer channel.
#[cfg_attr(feature = "trace", instrument(skip_all, fields(story_id = story.id)))]
async fn collect_story(client: Arc<ApiClient>, tx: Sender<ItemRef>, mut story: Item, rank: u64) {
    let story_id = story.id;
    debug!("Collecting comments for story_id {story_id}");

    // Collect all the nested comments for the story.
    let result = timeout(
        Duration::from_secs(60),
        send_comments(&client, story.id, mem::take(&mut story.kids), tx.clone()),
    )
    .await
    .map_err(|_| SearchError::TimedOut(format!("story_id {story_id}, sending comments")));

    if let Err(err) = result {
        error!("{err}");
    }

    if tx.is_closed() {
        error!("index writer channel is closed");
    }

    // Create the story document.
    if let Err(err) = tx.send(ItemRef::Story(StoryRef { story, rank })).await {
        error!("Failed to send story {err}");
    }
}

/// Get all stories and nested comments for the given category and send
/// each document to the index writer channel.
#[cfg_attr(feature = "trace", instrument(skip(tx)))]
async fn collect(
    tx: Sender<ItemRef>,
    category_type: ArticleType,
    mut progress_tx: mpsc::Sender<RebuildProgress>,
) -> SearchResult<()> {
    let client = api_client();
    let stories = client.articles(75, category_type).await?;
    if let Err(err) = progress_tx.try_send(RebuildProgress::Started(stories.len())) {
        error!("Failed to send progress status: {err}");
    }

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
                #[cfg(feature = "trace")]
                tokio::spawn(collect_story(client, tx, story, rank).in_current_span()),
                #[cfg(not(feature = "trace"))]
                tokio::spawn(collect_story(client, tx, story, rank)),
            )
            .map_err(move |_| SearchError::TimedOut(format!("collecting story: {story_id}")))
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(result) = handles.next().await {
        if let Err(err) = result.and_then(|r| Ok(r?)) {
            error!("Collect story failed: {err}");
        }

        if let Err(err) = progress_tx.try_send(RebuildProgress::StoryCompleted) {
            error!("Failed to send progress status: {err}");
        }
    }

    info!("Finished collecting stories");
    if let Err(err) = progress_tx.try_send(RebuildProgress::Completed) {
        error!("Failed to send progress status: {err}");
    }

    Ok(())
}

async fn write_items(
    mut rx: Receiver<ItemRef>,
    writer_context: &mut WriteContext<'_>,
) -> SearchResult<()> {
    while let Some(item) = rx.recv().await {
        match item {
            ItemRef::Story(s) => writer_context.write_story(s)?,
            ItemRef::Comment(c) => writer_context.write_comment(c)?,
        }
    }
    Ok(())
}

#[cfg_attr(feature = "trace", instrument(skip(ctx)))]
pub async fn rebuild_index(
    ctx: Arc<RwLock<SearchContext>>,
    category_type: ArticleType,
    progress_tx: mpsc::Sender<RebuildProgress>,
) -> SearchResult<IndexStats> {
    let start_time = Instant::now();
    info!("Creating index for {category_type}");

    let mut writer_context = ctx.read().unwrap().writer_context()?;
    writer_context.delete_all_docs()?;

    let (tx, rx) = channel::<ItemRef>(100);
    #[cfg(feature = "trace")]
    let result = tokio::spawn(collect(tx, category_type, progress_tx).in_current_span());
    #[cfg(not(feature = "trace"))]
    let result = tokio::spawn(collect(tx, category_type, progress_tx));

    let writing_result = write_items(rx, &mut writer_context).await;

    info!("Finished indexing");

    result.await.map_err(SearchError::Join).and_then(identity)?;
    writing_result?;

    writer_context.commit()?;

    let g = ctx.read().unwrap();
    g.refresh_reader()?;
    document_stats(&g, start_time.elapsed(), category_type)
}

#[derive(Debug, Clone, Copy)]
pub enum RebuildProgress {
    Started(usize),
    StoryCompleted,
    Completed,
}

pub async fn update_story(
    ctx: Arc<RwLock<SearchContext>>,
    story: Story,
) -> SearchResult<Option<Story>> {
    let api = api_client();
    let latest = api.item(story.id).await?;
    let story_id = story.id;

    Ok(if latest.descendants != Some(story.descendants) {
        info!(
            "New comments {}.. re-indexing story {story_id}",
            latest.descendants.unwrap_or_default()
        );

        let writer_context = ctx.read().unwrap().writer_context()?;
        rebuild_story(api, writer_context, &story, latest).await?;
        info!("Rebuilt story {story_id}");
        let g = ctx.read().unwrap();
        g.refresh_reader()?;
        Some(g.story(story_id)?)
    } else {
        None
    })
}

/// Re-index this story along with all it's nested comments. Comments
/// will be be fetched recursively and concurrently.
async fn rebuild_story(
    client: Arc<ApiClient>,
    mut writer_context: WriteContext<'_>,
    story: &Story,
    latest: Item,
) -> SearchResult<()> {
    writer_context.delete_story(story);
    let (tx, rx) = channel::<ItemRef>(100);

    let result = tokio::spawn(collect_story(client, tx, latest, story.rank));

    write_items(rx, &mut writer_context).await?;

    result.await?;
    writer_context.commit()?;
    Ok(())
}

/// Handles story server side event subscription and relays story updates
/// when necessary to the UI.
async fn handle_story_events(
    ctx: Arc<RwLock<SearchContext>>,
    client: Arc<ApiClient>,
    story: Story,
    mut ui_tx: mpsc::Sender<Story>,
    mut rx: Receiver<ItemEventData>,
) -> SearchResult<()> {
    let story_id = story.id;
    let mut current_story = story;

    while let Some(ItemEventData { data: latest, .. }) = rx.recv().await {
        if ui_tx.is_closed() {
            warn!("UI transmission channel is closed");
            break;
        }

        if latest.deleted || latest.dead {
            warn!(
                "Hmm this story {} has been deleted or is dead now",
                latest.id
            );
            break;
        }

        let latest_descendants = latest.descendants.unwrap_or_default();

        // We'll rebuild this story if either the number of comments or score has changed.
        if latest_descendants != current_story.descendants || latest.score != current_story.score {
            let writer_context = ctx.read().unwrap().writer_context()?;
            match rebuild_story(client.clone(), writer_context, &current_story, latest).await {
                Ok(_) => {
                    current_story.descendants = latest_descendants;
                    let new_story = {
                        let g = ctx.read().unwrap();
                        g.refresh_reader()?;
                        g.story(story_id)?
                    };
                    current_story.descendants = new_story.descendants;
                    current_story.score = new_story.score;

                    info!("Rebuilt story {story_id}");
                    if let Err(err) = ui_tx.send(new_story).await {
                        error!("Failed to notify UI of story event: {err}");
                    }
                }
                Err(err) => {
                    error!("Failed to update story event: {err}");
                }
            }
        }
    }

    info!("story event handler has terminated");
    Ok(())
}

async fn handle_comment_events(
    ctx: Arc<RwLock<SearchContext>>,
    client: Arc<ApiClient>,
    comment: Comment,
    mut ui_tx: mpsc::Sender<Comment>,
    mut rx: Receiver<ItemEventData>,
) -> SearchResult<()> {
    while let Some(item_event) = rx.recv().await {
        let mut comment = comment.clone();
        let (tx_comment, rx_comment) = channel(10);
        let story_id = comment.story_id;

        let child_ids = item_event.data.kids.clone();

        let client = client.clone();
        let result = tokio::spawn(async move {
            let client = client.clone();
            send_comments(&client, story_id, child_ids, tx_comment.clone()).await;
        });

        let mut writer_context = ctx.read().unwrap().writer_context()?;
        write_items(rx_comment, &mut writer_context).await?;

        result.await?;

        comment.kids = item_event.data.kids;
        if let Err(err) = ui_tx.send(comment).await {
            error!("Failed to send to ui: {err}");
            break;
        }
    }

    Ok(())
}

pub struct WatchState<const N: usize, EventData> {
    pub abort_handles: [AbortHandle; N],
    pub receiver: mpsc::Receiver<EventData>,
}

pub fn watch_story(
    ctx: Arc<RwLock<SearchContext>>,
    story: Story,
) -> SearchResult<WatchState<2, Story>> {
    let client = api_client();
    let (tx, rx) = channel(10);
    let (ui_tx, ui_rx) = mpsc::channel::<Story>(10);
    let story_id = story.id;
    let c = client.clone();

    Ok(WatchState {
        receiver: ui_rx,
        abort_handles: [
            tokio::spawn(async move {
                c.item_stream(story_id, tx)
                    .inspect_err(|err| error!("Failed to subscribe to story events: {err}"))
                    .await
            })
            .abort_handle(),
            tokio::spawn(
                handle_story_events(ctx.clone(), client.clone(), story, ui_tx, rx).inspect_err(
                    |err| {
                        error!("Story event handler encountered an error: {err}");
                    },
                ),
            )
            .abort_handle(),
        ],
    })
}

pub fn watch_comment(
    ctx: Arc<RwLock<SearchContext>>,
    comment: Comment,
) -> SearchResult<WatchState<2, Comment>> {
    let client = api_client();
    let (tx, rx) = channel(10);
    let (ui_tx, ui_rx) = mpsc::channel::<Comment>(10);
    let comment_id = comment.id;
    let c = client.clone();

    Ok(WatchState {
        receiver: ui_rx,
        abort_handles: [
            tokio::spawn(async move {
                c.item_stream(comment_id, tx)
                    .inspect_err(|err| error!("Failed to subscribe to item events: {err}"))
                    .await
            })
            .abort_handle(),
            tokio::spawn(
                handle_comment_events(ctx, client, comment, ui_tx, rx)
                    .inspect_err(|err| error!("Comment event handler encountered an error: {err}")),
            )
            .abort_handle(),
        ],
    })
}

pub fn document_stats(
    ctx: &SearchContext,
    build_time: Duration,
    category: ArticleType,
) -> SearchResult<IndexStats> {
    let searcher = if ctx.active_index == category {
        ctx.searcher()
    } else {
        ctx.indices.get_index(category).reader()?.searcher()
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
