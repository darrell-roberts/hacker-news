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
    InteractiveElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled,
    StyledText, TextRun, UnderlineStyle, Window,
};
use hacker_news_api::Item;
use html_sanitizer::{parse_elements, Element};
use log::error;
use std::{collections::HashMap, sync::Arc, time::Duration};

pub struct CommentView {
    text: SharedString,
    author: SharedString,
    children: HashMap<u64, Entity<CommentView>>,
    comment_ids: Arc<Vec<u64>>,
    comment_image: ImageSource,
    total_comments: SharedString,
    loading_comments: bool,
    article: Entity<ArticleView>,
    text_layout: Vec<TextLayout>,
    age: SharedString,
}

impl CommentView {
    pub fn new(
        cx: &mut AsyncApp,
        item: Item,
        article: Entity<ArticleView>,
    ) -> anyhow::Result<Entity<Self>> {
        let (text, text_layout) = item
            .text
            .as_deref()
            .map(parse_elements)
            .map(text_layout)
            .unwrap_or_default();

        cx.new(|_cx| Self {
            text: text.into(),
            author: format!("by: {} ({})", item.by, item.id).into(),
            children: HashMap::new(),
            total_comments: format!("{}", item.kids.len()).into(),
            comment_ids: Arc::new(item.kids),
            comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
            loading_comments: false,
            article,
            text_layout,
            age: parse_date(item.time).unwrap_or_default().into(),
        })
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
        let weak_entity = cx.weak_entity();

        let article = self.article.clone();
        let close_comment = cx.weak_entity();

        div()
            .bg(theme.surface())
            .rounded_md()
            .border_1()
            .border_color(theme.border())
            .shadow_sm()
            .text_color(theme.text_color())
            .m_1()
            .p_1()
            .child(
                div().pb_2().child(
                    StyledText::new(self.text.clone())
                        .with_runs(rich_text_runs(theme, self.text_layout.clone()).collect()),
                ),
            )
            .child(
                gpui::div()
                    .flex()
                    .flex_row()
                    .italic()
                    .gap_1()
                    .border_t_1()
                    .pt_1()
                    .border_color(theme.border())
                    .text_size(rems(0.75))
                    .child(self.author.clone())
                    .child(self.age.clone())
                    .when(!self.comment_ids.is_empty(), |el| {
                        el.child(
                            div()
                                .id("child-comments")
                                .cursor_pointer()
                                .on_click(move |_event, _window, app| {
                                    if let Err(err) = weak_entity.update(app, |this, _cx| {
                                        this.loading_comments = true;
                                    }) {
                                        error!("Failed to update loading status: {err}");
                                    };
                                    let ids = ids.clone();
                                    let weak_entity = weak_entity.clone();
                                    let article = article.clone();
                                    app.spawn(async move |async_app| {
                                        let client = async_app
                                            .read_global(|client: &ApiClientState, _| {
                                                client.0.clone()
                                            })
                                            .unwrap();
                                        let items = async_compat::Compat::new(
                                            client.items(&ids).try_collect::<Vec<_>>(),
                                        )
                                        .await
                                        .unwrap_or_default();
                                        let children = items
                                            .into_iter()
                                            .filter_map(|item| {
                                                let id = item.id;
                                                CommentView::new(async_app, item, article.clone())
                                                    .ok()
                                                    .map(|view| (id, view))
                                            })
                                            .collect::<HashMap<_, _>>();
                                        if let Err(err) =
                                            weak_entity.update(async_app, |this, _cx| {
                                                this.children = children;
                                                this.loading_comments = false;
                                            })
                                        {
                                            error!("Failed to update child comments: {err}");
                                        };
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
                    }),
            )
            .when(!self.children.is_empty(), |el| {
                el.child(
                    div()
                        .border_1()
                        .bg(theme.bg())
                        .border_color(theme.border())
                        .p_1()
                        .m_1()
                        .shadow_sm()
                        .child(
                            div()
                                .flex()
                                .flex_grow()
                                .flex_row()
                                .justify_end()
                                .child("[X]")
                                .cursor_pointer()
                                .id("close-comments")
                                .on_click(move |_event, _window, app| {
                                    if let Some(this) = close_comment.upgrade() {
                                        this.update(app, |comment_view, _cx| {
                                            comment_view.children.clear();
                                        });
                                    }
                                }),
                        )
                        .children(self.children.values().cloned()),
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
        // background_color: Some(rgb(0xeaeaea).into()),
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
        // background_color: Some(rgb(0xe6e600).into()),
        background_color: None,
        underline: Some(UnderlineStyle {
            thickness: px(1.),
            color: Some(theme.text_color().into()),
            wavy: true,
        }),
        strikethrough: None,
    }
}

fn text_layout(elements: Vec<Element<'_>>) -> (String, Vec<TextLayout>) {
    let mut layout = Vec::new();
    let mut buffer = String::new();

    for element in elements {
        match element {
            Element::Text(s) => {
                layout.push(TextLayout::Normal(s.len()));
                buffer.push_str(s);
            }
            Element::Link(anchor) => {
                let text = anchor
                    .attributes
                    .iter()
                    .find(|a| a.name == "href")
                    .map(|attr| {
                        if anchor.children.is_empty() {
                            attr.value.as_str()
                        } else {
                            anchor.children.as_str()
                        }
                    });

                if let Some(text) = text {
                    layout.push(TextLayout::Link(text.len()));
                    buffer.push_str(text);
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
                let (italic_str, _ls) = text_layout(elements);
                layout.push(TextLayout::Italic(italic_str.len()));
                buffer.push_str(&italic_str);
            }
            Element::Bold(elements) => {
                let (bold_str, _ls) = text_layout(elements);
                layout.push(TextLayout::Bold(bold_str.len()));
                buffer.push_str(&bold_str);
            }
        }
    }

    (buffer, layout)
}

#[derive(Copy, Clone)]
enum TextLayout {
    Normal(usize),
    Bold(usize),
    Italic(usize),
    Link(usize),
    Code(usize),
}

fn rich_text_runs(theme: Theme, layout: Vec<TextLayout>) -> impl Iterator<Item = TextRun> {
    layout.into_iter().map(move |element| match element {
        TextLayout::Normal(n) => normal(theme, n),
        TextLayout::Bold(n) => bold(theme, n),
        TextLayout::Italic(n) => italic(theme, n),
        TextLayout::Link(n) => link(theme, n),
        TextLayout::Code(n) => code(theme, n),
    })
}
