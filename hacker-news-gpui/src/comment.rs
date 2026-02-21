//! Render comment
use crate::{
    article::ArticleView,
    common::{parse_date, COMMENT_IMAGE},
    theme::Theme,
    ApiClientState,
};
use futures::TryStreamExt as _;
use gpui::{
    div, img, prelude::FluentBuilder as _, pulsating_between, px, rems, Animation,
    AnimationExt as _, AppContext as _, AsyncApp, Entity, Font, FontWeight, ImageSource,
    InteractiveElement, InteractiveText, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, StyledText, TextRun, UnderlineStyle, Window,
};
use hacker_news_api::Item;
use html_sanitizer::{parse_elements, Element};
use std::{ops::Range, sync::Arc, time::Duration};

/// Comment view with state.
pub struct CommentView {
    text: SharedString,
    author: SharedString,
    children: Vec<Entity<CommentView>>,
    comment_ids: Arc<Vec<u64>>,
    comment_image: ImageSource,
    total_comments: SharedString,
    loading_comments: bool,
    article: Entity<ArticleView>,
    text_layout: Vec<TextLayout>,
    age: SharedString,
    urls: Vec<String>,
}

impl CommentView {
    /// Create a new comment view
    ///
    /// # Arguments
    ///
    /// * `cx` - The async application context used to create new entities.
    /// * `item` - The Hacker News API item representing the comment.
    /// * `article` - The entity representing the parent article view.
    pub fn new(cx: &mut AsyncApp, item: Item, article: Entity<ArticleView>) -> Entity<Self> {
        let ParsedComment { text, layout, urls } = item
            .text
            .as_deref()
            .map(parse_elements)
            .map(parse_layout)
            .unwrap_or_default();

        cx.new(|_cx| Self {
            text: text.into(),
            author: format!("by: {} ({})", item.by, item.id).into(),
            children: Vec::new(),
            total_comments: format!("{}", item.kids.len()).into(),
            comment_ids: Arc::new(item.kids),
            comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
            loading_comments: false,
            article,
            text_layout: layout,
            urls,
            age: parse_date(item.time).unwrap_or_default().into(),
        })
    }

    /// Renders the comment text.
    ///
    /// # Arguments
    ///
    /// * `theme` - The current theme used for styling.
    /// * `comment_entity` - The entity representing this comment view.
    fn render_text_area(&self, theme: Theme, comment_entity: Entity<CommentView>) -> gpui::Div {
        div().p_1().child(
            InteractiveText::new(
                "comment_text",
                StyledText::new(self.text.clone())
                    .with_runs(rich_text_runs(theme, &self.text_layout).collect()),
            )
            .on_click(url_ranges(&self.text_layout), move |index, _window, app| {
                comment_entity.read_with(app, |this: &CommentView, app| {
                    if let Some(url) = this.urls.get(index) {
                        app.open_url(url);
                    }
                })
            }),
        )
    }

    /// Renders the comment footer with child comment count, author and date/time.
    ///
    /// # Arguments
    ///
    /// * `theme` - The current theme used for styling.
    /// * `ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    fn render_comment_footer(
        &self,
        theme: Theme,
        ids: Arc<Vec<u64>>,
        comment_entity: Entity<CommentView>,
    ) -> gpui::Div {
        gpui::div()
            .flex()
            .flex_row()
            .italic()
            .gap_1()
            .border_t_1()
            .p_1()
            .border_color(theme.border())
            .text_size(rems(0.75))
            .child(self.author.clone())
            .child(self.age.clone())
            .when(!self.comment_ids.is_empty(), |div| {
                self.render_child_comments(ids, comment_entity, div)
            })
    }

    /// Render child comments that have opened.
    ///
    /// Render child comments that have opened.
    ///
    /// # Arguments
    ///
    /// * `ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    /// * `el` - The parent Div element to which child comments UI will be attached.
    fn render_child_comments(
        &self,
        ids: Arc<Vec<u64>>,
        comment_entity: Entity<CommentView>,
        el: gpui::Div,
    ) -> gpui::Div {
        let article_entity = self.article.clone();

        el.child(
            div()
                .id("child-comments")
                .cursor_pointer()
                .on_click(move |_event, _window, app| {
                    comment_entity.update(app, |this, _cx| {
                        this.loading_comments = true;
                    });

                    let ids = ids.clone();
                    let comment_entity = comment_entity.clone();
                    let article_entity = article_entity.clone();

                    app.spawn(async move |async_app| {
                        let client =
                            async_app.read_global(|client: &ApiClientState, _| client.0.clone());
                        let items =
                            async_compat::Compat::new(client.items(&ids).try_collect::<Vec<_>>())
                                .await
                                .unwrap_or_default();
                        let children = items
                            .into_iter()
                            .map(|item| CommentView::new(async_app, item, article_entity.clone()))
                            .collect::<Vec<_>>();
                        comment_entity.update(async_app, |this, _cx| {
                            this.children = children;
                            this.loading_comments = false;
                        });
                    })
                    .detach();
                })
                .flex()
                .flex_row()
                .child(self.total_comments.clone())
                .child(div().child(img(self.comment_image.clone())).when(
                    self.loading_comments,
                    |el| {
                        gpui::div().child(
                            el.with_animation(
                                "comment-loading",
                                Animation::new(Duration::from_secs(1))
                                    .repeat()
                                    .with_easing(pulsating_between(0.1, 0.8)),
                                |label, delta| label.opacity(delta),
                            ),
                        )
                    },
                )),
        )
    }
}

impl Render for CommentView {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        let theme: Theme = window.appearance().into();

        let ids = self.comment_ids.clone();
        let comment_entity = cx.entity();

        div()
            .bg(theme.surface())
            .rounded_tl_md()
            .mt_1()
            .child(self.render_text_area(theme, comment_entity.clone()))
            .child(self.render_comment_footer(theme, ids, comment_entity.clone()))
            .when(!self.children.is_empty(), |el| {
                el.child(
                    div()
                        .bg(theme.comment_border())
                        .pl_1()
                        .ml_1()
                        .rounded_tl_md()
                        .child(
                            div()
                                .flex()
                                .flex_grow()
                                .flex_row()
                                .text_size(rems(0.75))
                                .child("[X]")
                                .cursor_pointer()
                                .id("close-comments")
                                .on_click(move |_event, _window, app| {
                                    comment_entity.update(app, |comment_view, _cx| {
                                        comment_view.children.clear();
                                    });
                                }),
                        )
                        .children(self.children.clone()),
                )
            })
    }
}

fn normal(theme: Theme, len: usize) -> TextRun {
    TextRun {
        len,
        font: Font {
            family: SharedString::new(".SystemUIFont"),
            features: Default::default(),
            fallbacks: None,
            weight: Default::default(),
            style: Default::default(),
        },
        color: theme.text_color().into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

fn italic(theme: Theme, len: usize) -> TextRun {
    TextRun {
        len,
        font: Font {
            family: SharedString::new(".SystemUIFont"),
            features: Default::default(),
            fallbacks: None,
            weight: Default::default(),
            style: gpui::FontStyle::Italic,
        },
        color: theme.text_color().into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

fn bold(theme: Theme, len: usize) -> TextRun {
    TextRun {
        len,
        font: Font {
            family: SharedString::new(".SystemUIFont"),
            features: Default::default(),
            fallbacks: None,
            weight: FontWeight::BOLD,
            style: gpui::FontStyle::Normal,
        },
        color: theme.text_color().into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

fn code(theme: Theme, len: usize) -> TextRun {
    TextRun {
        len,
        font: Font {
            family: SharedString::new("Menlo"),
            features: Default::default(),
            fallbacks: None,
            weight: Default::default(),
            style: gpui::FontStyle::Normal,
        },
        color: theme.text_color().into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

fn link(theme: Theme, len: usize) -> TextRun {
    TextRun {
        len,
        font: Font {
            family: SharedString::new(".SystemUIFont"),
            features: Default::default(),
            fallbacks: None,
            weight: FontWeight::default(),
            style: gpui::FontStyle::Normal,
        },
        color: theme.text_color().into(),
        background_color: None,
        underline: Some(UnderlineStyle {
            thickness: px(1.),
            color: Some(theme.text_color().into()),
            wavy: true,
        }),
        strikethrough: None,
    }
}

#[derive(Default)]
struct ParsedComment {
    text: String,
    layout: Vec<TextLayout>,
    urls: Vec<String>,
}

fn parse_layout(elements: Vec<Element<'_>>) -> ParsedComment {
    let mut layout = Vec::new();
    let mut buffer = String::new();
    let mut urls = Vec::new();

    for element in elements {
        match element {
            Element::Text(s) => {
                layout.push(TextLayout::Normal(s.len()));
                buffer.push_str(s);
            }
            Element::Link(anchor) => {
                let link = anchor
                    .attributes
                    .iter()
                    .find(|a| a.name == "href")
                    .map(|attr| {
                        if anchor.children.is_empty() {
                            (attr.value.as_str(), attr.value.as_str())
                        } else {
                            (anchor.children.as_str(), attr.value.as_str())
                        }
                    });

                if let Some((text, url)) = link {
                    layout.push(TextLayout::Link(text.len()));
                    buffer.push_str(text);
                    urls.push(url.to_string());
                }
            }
            Element::Escaped(c) => {
                layout.push(TextLayout::Normal(1));
                buffer.push(c);
            }
            Element::Paragraph => {
                layout.push(TextLayout::Normal(1));
                buffer.push('\n');
            }
            Element::Code(s) => {
                layout.push(TextLayout::Code(s.len()));
                buffer.push_str(&s);
            }
            Element::Italic(elements) => {
                let ParsedComment { text, .. } = parse_layout(elements);
                layout.push(TextLayout::Italic(text.len()));
                buffer.push_str(&text);
            }
            Element::Bold(elements) => {
                let ParsedComment { text, .. } = parse_layout(elements);
                layout.push(TextLayout::Bold(text.len()));
                buffer.push_str(&text);
            }
        }
    }

    ParsedComment {
        text: buffer,
        layout,
        urls,
    }
}

#[derive(Copy, Clone)]
enum TextLayout {
    Normal(usize),
    Bold(usize),
    Italic(usize),
    Link(usize),
    Code(usize),
}

fn rich_text_runs(theme: Theme, layout: &[TextLayout]) -> impl Iterator<Item = TextRun> + use<'_> {
    layout.iter().map(move |element| match element {
        TextLayout::Normal(n) => normal(theme, *n),
        TextLayout::Bold(n) => bold(theme, *n),
        TextLayout::Italic(n) => italic(theme, *n),
        TextLayout::Link(n) => link(theme, *n),
        TextLayout::Code(n) => code(theme, *n),
    })
}

fn url_ranges(layout: &[TextLayout]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut total_chars = 0;
    for l in layout {
        match l {
            TextLayout::Normal(n) => {
                total_chars += n;
            }
            TextLayout::Bold(n) => {
                total_chars += n;
            }
            TextLayout::Italic(n) => {
                total_chars += n;
            }
            TextLayout::Link(n) => {
                ranges.push(total_chars..(total_chars + n));
                total_chars += n;
            }
            TextLayout::Code(n) => {
                total_chars += n;
            }
        }
    }
    ranges
}
