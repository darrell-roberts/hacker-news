use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_CATEGORY, ITEM_DESCENDANT_COUNT, ITEM_ID,
    ITEM_KIDS, ITEM_PARENT_ID, ITEM_RANK, ITEM_SCORE, ITEM_STORY_ID, ITEM_TIME, ITEM_TITLE,
    ITEM_TYPE, ITEM_URL,
};
use hacker_news_api::{ApiClient, Item};
use std::{future::Future, pin::Pin};
use tantivy::{IndexWriter, TantivyDocument, Term};

pub fn index_articles<'a>(
    ctx: &'a SearchContext,
    client: &'a ApiClient,
    writer: &'a mut IndexWriter,
    items: &'a [Item],
    category: &'a str,
    mut story_id: Option<u64>,
) -> Pin<Box<impl Future<Output = Result<(), SearchError>> + use<'a>>> {
    Box::pin(async move {
        let id = ctx.schema.get_field(ITEM_ID)?;
        let parent = ctx.schema.get_field(ITEM_PARENT_ID)?;
        let title = ctx.schema.get_field(ITEM_TITLE)?;
        let body = ctx.schema.get_field(ITEM_BODY)?;
        let url = ctx.schema.get_field(ITEM_URL)?;
        let by = ctx.schema.get_field(ITEM_BY)?;
        let ty = ctx.schema.get_field(ITEM_TYPE)?;
        let rank = ctx.schema.get_field(ITEM_RANK)?;
        let descendant_count = ctx.schema.get_field(ITEM_DESCENDANT_COUNT)?;
        let category_field = ctx.schema.get_field(ITEM_CATEGORY)?;
        let time = ctx.schema.get_field(ITEM_TIME)?;
        let parent_story_id = ctx.schema.get_field(ITEM_STORY_ID)?;
        let kids = ctx.schema.get_field(ITEM_KIDS)?;
        let score = ctx.schema.get_field(ITEM_SCORE)?;

        for (item, index) in items.iter().zip(1..) {
            let mut doc = TantivyDocument::new();
            doc.add_u64(rank, index);
            doc.add_u64(id, item.id);
            if let Some(id) = item.parent {
                doc.add_u64(parent, id);
            }
            if let Some(t) = item.title.as_deref() {
                doc.add_text(title, t);
            }
            if let Some(t) = item.text.as_deref() {
                doc.add_text(body, t);
            }
            if let Some(u) = item.url.as_deref() {
                doc.add_text(url, u);
            }
            doc.add_text(by, &item.by);
            doc.add_text(ty, &item.ty);

            if let Some(n) = item.descendants {
                doc.add_u64(descendant_count, n);
            }

            if let Some(id) = story_id {
                doc.add_u64(parent_story_id, id);
            }

            if item.ty == "story" {
                story_id = Some(item.id);
                doc.add_text(category_field, category);
                doc.add_u64(score, item.score);
            }

            doc.add_u64(time, item.time);

            for id in &item.kids {
                doc.add_u64(kids, *id);
            }

            if !item.kids.is_empty() {
                let children = client.items(&item.kids).await?;
                index_articles(ctx, client, writer, &children, category, story_id).await?;
            }

            // TODO: Add a depth field for comments. This will allow for
            // easy indentatino for viewing the comment tree.

            writer.add_document(doc)?;
        }
        Ok(())
    })
}

pub async fn rebuild_index(ctx: &SearchContext) -> Result<(u64, u64), SearchError> {
    let client = ApiClient::new()?;

    let articles = client
        .articles(75, hacker_news_api::ArticleType::Top)
        .await?;

    let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;
    writer.delete_all_documents()?;

    index_articles(ctx, &client, &mut writer, &articles, "top", None).await?;
    writer.commit()?;

    ctx.reader.reload()?;

    document_stats(ctx)
}

pub fn document_stats(ctx: &SearchContext) -> Result<(u64, u64), SearchError> {
    let searcher = ctx.searcher();
    let total_comments = searcher.doc_freq(&Term::from_field_text(
        ctx.schema.get_field(ITEM_TYPE)?,
        "comment",
    ))?;

    let total_documents = searcher.num_docs();
    Ok((total_documents, total_comments))
}
