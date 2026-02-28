//! Render comment
use crate::{
    article::ArticleView,
    common::{COMMENT_IMAGE, comment_entities, parse_date},
    theme::Theme,
};
use gpui::{
    Animation, AnimationExt as _, AppContext as _, AsyncApp, Entity, Font, FontWeight, ImageSource,
    InteractiveElement, InteractiveText, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, StyledText, TextRun, UnderlineStyle, Window, div, img,
    prelude::FluentBuilder as _, pulsating_between, px, rems,
};
use hacker_news_api::Item;
use html_sanitizer::{Element, parse_elements};
use std::{ops::Range, sync::Arc, time::Duration};

/// Comment view with state.
pub struct CommentView {
    text: SharedString,
    /// The author of the article, formatted as "by {author}".
    author: SharedString,
    /// The entities representing the child comments for this comment.
    children: Vec<Entity<CommentView>>,
    /// The ids of child comments.
    comment_child_ids: Arc<Vec<u64>>,
    /// The image source for the comment icon.
    comment_image: ImageSource,
    /// The number of comments on the comment, if available.
    comment_count: SharedString,
    /// Whether the comments are currently loading.
    loading_comments: bool,
    /// The top level article entity this comment is a descendant of.
    article_entity: Entity<ArticleView>,
    /// Text layout structure of the comment body for rendering as [`StyledText`].
    text_layout: Vec<TextLayout>,
    /// The age of the comment, formatted as a string.
    age: SharedString,
    /// Any urls that are in the comment body.
    urls: Vec<String>,
}

impl CommentView {
    /// Create a new comment view
    ///
    /// # Arguments
    ///
    /// * `cx` - The async application context used to create new entities.
    /// * `item` - The Hacker News API item representing the comment.
    /// * `article_entity` - The entity representing the parent article view.
    pub fn new(cx: &mut AsyncApp, item: Item, article_entity: Entity<ArticleView>) -> Entity<Self> {
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
            comment_count: format!("{}", item.kids.len()).into(),
            comment_child_ids: Arc::new(item.kids),
            comment_image: ImageSource::Image(Arc::clone(&COMMENT_IMAGE)),
            loading_comments: false,
            article_entity,
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
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the rendered comment text area UI.
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
    /// * `comment_ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the comment footer UI.
    fn render_comment_footer(
        &self,
        theme: Theme,
        comment_ids: Arc<Vec<u64>>,
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
            .when(!self.comment_child_ids.is_empty(), |div| {
                self.render_child_comments(comment_ids, comment_entity, div)
            })
    }

    /// Render child comments that have opened.
    ///
    /// # Arguments
    ///
    /// * `comment_ids` - The list of child comment IDs.
    /// * `comment_entity` - The entity representing this comment view.
    /// * `el` - The parent Div element to which child comments UI will be attached.
    ///
    /// # Returns
    ///
    /// Returns a `gpui::Div` element containing the child comments UI.
    fn render_child_comments(
        &self,
        comment_ids: Arc<Vec<u64>>,
        comment_entity: Entity<CommentView>,
        el: gpui::Div,
    ) -> gpui::Div {
        let article_entity = self.article_entity.clone();

        el.child(
            div()
                .id("child-comments")
                .cursor_pointer()
                .on_click(move |_event, _window, app| {
                    comment_entity.update(app, |this, _cx| {
                        this.loading_comments = true;
                    });

                    let comment_ids = comment_ids.clone();
                    let comment_entity = comment_entity.clone();
                    let article_entity = article_entity.clone();

                    app.spawn(async move |async_app| {
                        let comment_entities =
                            comment_entities(async_app, article_entity, &comment_ids).await;
                        comment_entity.update(async_app, |this, _cx| {
                            this.children = comment_entities;
                            this.loading_comments = false;
                        });
                    })
                    .detach();
                })
                .flex()
                .flex_row()
                .child(self.comment_count.clone())
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

        let comment_ids = self.comment_child_ids.clone();
        let comment_entity = cx.entity();

        div()
            .bg(theme.surface())
            .rounded_tl_md()
            .flex_col()
            .w_full()
            .mt_1()
            .child(self.render_text_area(theme, comment_entity.clone()))
            .child(self.render_comment_footer(theme, comment_ids, comment_entity.clone()))
            .when(!self.children.is_empty(), |el| {
                el.child(
                    div()
                        .bg(theme.comment_border())
                        .pl_1()
                        .ml_1()
                        .w_full()
                        .flex_col()
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

/// Creates a `TextRun` representing normal text with the given length.
/// It uses the system UI font and the current theme's text color.
///
/// # Arguments:
///
///   * theme: The current theme used for styling.
///   * len: The length of the text run.
///
/// # Returns:
///
///   A `TextRun` configured for normal text.
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

/// Creates a `TextRun` representing italic text with the given length.
/// It uses the system UI font with italic style and the current theme's text color.
///
/// # Arguments:
///
///   * theme: The current theme used for styling.
///   * len: The length of the text run.
///
/// # Returns:
///
///   A `TextRun` configured for italic text.
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

/// Creates a `TextRun` representing bold text with the given length.
/// It uses the system UI font with bold weight and the current theme's text color.
///
/// # Arguments:
///
///   * theme: The current theme used for styling.
///   * len: The length of the text run.
///
/// # Returns:
///
///   A `TextRun` configured for bold text.
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

/// Creates a `TextRun` representing monospaced code text with the given length.
/// It uses the Menlo font and the current theme's text color.
///
/// # Arguments:
///
///   * theme: The current theme used for styling.
///   * len: The length of the text run.
///
/// # Returns:
///
///   A `TextRun` configured for code text.
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

/// Creates a `TextRun` representing a clickable link with the given length.
/// It uses the system UI font with a wavy underline and the current theme's text color.
///
/// # Arguments:
///
///   * theme: The current theme used for styling.
///   * len: The length of the text run.
///
/// # Returns:
///
///   A `TextRun` configured for link text.
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
/// A comment body with layout and url properties
/// for formatting as [`StyledText`].
struct ParsedComment {
    /// Full comment body as text
    text: String,
    /// Layouts to format the comment body by character index.
    layout: Vec<TextLayout>,
    /// Url strings for rendering links.
    urls: Vec<String>,
}

/// Parses a list of HTML [`Element`]s into a [`ParsedComment`], extracting
/// the plain text content, text layout information for rich formatting,
/// and any URLs found in link elements.
///
/// # Arguments
///
/// * `elements` - A vector of parsed HTML elements representing the comment body.
///
/// # Returns
///
/// A [`ParsedComment`] containing the extracted text, layout metadata, and URLs.
fn parse_layout(elements: Vec<Element<'_>>) -> ParsedComment {
    let mut parsed = ParsedComment::default();
    collect_elements(&elements, &mut parsed, None);
    parsed
}

/// Recursively collects elements into the shared `ParsedComment` accumulator.
/// When `wrapper` is `Some`, the inner text segments are wrapped in that
/// layout style (e.g. `Italic` or `Bold`) instead of their own default.
/// Links inside a wrapper still produce `Link` layout entries so that URLs
/// are never lost.
fn collect_elements(
    elements: &[Element<'_>],
    parsed: &mut ParsedComment,
    wrapper: Option<fn(usize) -> TextLayout>,
) {
    for element in elements {
        match element {
            Element::Text(s) => {
                let layout_fn = wrapper.unwrap_or(TextLayout::Normal);
                push_or_merge(&mut parsed.layout, layout_fn, s.len());
                parsed.text.push_str(s);
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
                    parsed.layout.push(TextLayout::Link(text.len()));
                    parsed.text.push_str(text);
                    parsed.urls.push(url.to_string());
                }
            }
            Element::Escaped(c) => {
                let layout_fn = wrapper.unwrap_or(TextLayout::Normal);
                push_or_merge(&mut parsed.layout, layout_fn, 1);
                parsed.text.push(*c);
            }
            Element::Paragraph => {
                push_or_merge(&mut parsed.layout, TextLayout::Normal, 1);
                parsed.text.push('\n');
            }
            Element::Code(s) => {
                parsed.layout.push(TextLayout::Code(s.len()));
                parsed.text.push_str(s);
            }
            Element::Italic(children) => {
                collect_elements(children, parsed, Some(TextLayout::Italic));
            }
            Element::Bold(children) => {
                collect_elements(children, parsed, Some(TextLayout::Bold));
            }
        }
    }
}

/// Pushes a new layout entry or extends the last one if it is the same variant,
/// reducing fragmentation from many small adjacent segments of the same style.
fn push_or_merge(layout: &mut Vec<TextLayout>, ctor: fn(usize) -> TextLayout, len: usize) {
    // Check if the last entry is the same variant so we can merge.
    if let Some(last) = layout.last_mut() {
        // Build a dummy to compare discriminants.
        let candidate = ctor(0);
        if std::mem::discriminant(last) == std::mem::discriminant(&candidate) {
            *last = ctor(last.len() + len);
            return;
        }
    }
    layout.push(ctor(len));
}

/// Represents the different text formatting styles used in comment body layout.
/// Each variant holds the length (in characters) of the text segment it applies to.
#[derive(Copy, Clone)]
enum TextLayout {
    /// Normal, unstyled text.
    Normal(usize),
    /// Bold text.
    Bold(usize),
    /// Italic text.
    Italic(usize),
    /// A clickable link.
    Link(usize),
    /// Monospaced code text.
    Code(usize),
}

impl TextLayout {
    /// Returns the character length of this text layout segment.
    fn len(&self) -> usize {
        match self {
            TextLayout::Normal(n)
            | TextLayout::Bold(n)
            | TextLayout::Italic(n)
            | TextLayout::Link(n)
            | TextLayout::Code(n) => *n,
        }
    }
}

/// Converts a slice of `TextLayout` elements into an iterator of `TextRun` items,
/// applying the appropriate text styling (normal, bold, italic, link, or code)
/// based on each layout element's variant.
///
/// # Arguments
///
///   * theme: The current theme used for styling the text runs.
///   * layout: A slice of `TextLayout` elements describing the formatting of each text segment.
///
/// # Returns
///
///   An iterator of `TextRun` items corresponding to the styled text segments.
fn rich_text_runs(theme: Theme, layout: &[TextLayout]) -> impl Iterator<Item = TextRun> + use<'_> {
    layout.iter().map(move |element| match element {
        TextLayout::Normal(n) => normal(theme, *n),
        TextLayout::Bold(n) => bold(theme, *n),
        TextLayout::Italic(n) => italic(theme, *n),
        TextLayout::Link(n) => link(theme, *n),
        TextLayout::Code(n) => code(theme, *n),
    })
}

/// Computes the character ranges of all link segments within the given text layouts.
/// These ranges are used to identify clickable regions in the rendered comment text.
///
/// # Arguments
///
///   * layouts: A slice of `TextLayout` elements describing the formatting of each text segment.
///
/// # Returns
///
///   A vector of `Range<usize>` values, each representing the character range of a link segment.
fn url_ranges(layouts: &[TextLayout]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut total_chars = 0;
    for layout in layouts {
        let n = layout.len();
        if matches!(layout, TextLayout::Link(_)) {
            ranges.push(total_chars..(total_chars + n));
        }
        total_chars += n;
    }
    ranges
}
