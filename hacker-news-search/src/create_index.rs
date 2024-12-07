use hacker_news_api::{ApiClient, Item};
use std::{future::Future, pin::Pin};
use tantivy::{IndexWriter, TantivyDocument};

use crate::{
    SearchContext, SearchError, ITEM_BODY, ITEM_BY, ITEM_CATEGORY, ITEM_DESCENDANT_COUNT, ITEM_ID,
    ITEM_PARENT_ID, ITEM_RANK, ITEM_TIME, ITEM_TITLE, ITEM_TYPE, ITEM_URL,
};

pub fn index_articles<'a>(
    ctx: &'a SearchContext,
    client: &'a ApiClient,
    writer: &'a mut IndexWriter,
    articles: &'a [Item],
    category: &'a str,
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

        for (article, index) in articles.iter().zip(1..) {
            let mut doc = TantivyDocument::new();
            doc.add_u64(rank, index);
            doc.add_u64(id, article.id);
            if let Some(id) = article.parent {
                doc.add_u64(parent, id);
            }
            if let Some(t) = article.title.as_deref() {
                doc.add_text(title, t);
            }
            if let Some(t) = article.text.as_deref() {
                doc.add_text(body, t);
            }
            if let Some(u) = article.url.as_deref() {
                doc.add_text(url, u);
            }
            doc.add_text(by, &article.by);
            doc.add_text(ty, &article.ty);

            if !article.kids.is_empty() {
                let children = client.items(&article.kids).await?;
                index_articles(ctx, client, writer, &children, category).await?;
            }

            if let Some(n) = article.descendants {
                doc.add_u64(descendant_count, n);
            }

            if article.ty == "story" {
                doc.add_text(category_field, category);
            }

            doc.add_u64(time, article.time);

            writer.add_document(doc)?;
        }
        Ok(())
    })
}

pub async fn create_index(ctx: &SearchContext) -> Result<(), SearchError> {
    let client = ApiClient::new()?;

    let articles = client
        .articles(25, hacker_news_api::ArticleType::Top)
        .await?;

    let mut writer: IndexWriter = ctx.index.writer(50_000_000)?;
    index_articles(ctx, &client, &mut writer, &articles, "top").await?;
    writer.commit()?;

    Ok(())
}
