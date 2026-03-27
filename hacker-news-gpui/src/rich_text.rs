//! Convert parsed hacker news rich text into gpui TextRun.
use crate::{common::url_punycode, theme::Theme};
use gpui::{Font, FontWeight, SharedString, TextRun, UnderlineStyle, px};
use html_sanitizer::{Element, parse_elements};
use std::{borrow::Cow, ops::Range};

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
            family: SharedString::new("Courier"),
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
pub struct ParsedStyledText {
    /// Full comment body as text
    pub text: String,
    /// Layouts to format the comment body by character index.
    pub layout: Vec<TextLayout>,
    /// Url strings for rendering links.
    pub urls: Vec<String>,
}

pub struct ViewStyledText {
    /// Full text
    pub text: SharedString,
    /// Layouts to format the comment body by character index.
    pub layout: Vec<TextLayout>,
    /// Url strings for rendering links.
    pub urls: Vec<String>,
}

impl From<ParsedStyledText> for ViewStyledText {
    fn from(parsed_styled_text: ParsedStyledText) -> Self {
        Self {
            text: parsed_styled_text.text.into(),
            layout: parsed_styled_text.layout,
            urls: parsed_styled_text.urls,
        }
    }
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
pub fn parse_layout(text: &str) -> ParsedStyledText {
    let mut parsed = ParsedStyledText::default();
    collect_elements(parse_elements(text), &mut parsed, None);
    parsed
}

/// Recursively collects elements into the shared `ParsedComment` accumulator.
/// When `wrapper` is `Some`, the inner text segments are wrapped in that
/// layout style (e.g. `Italic` or `Bold`) instead of their own default.
/// Links inside a wrapper still produce `Link` layout entries so that URLs
/// are never lost.
fn collect_elements(
    elements: Vec<Element<'_>>,
    parsed: &mut ParsedStyledText,
    wrapper: Option<fn(usize) -> TextLayout>,
) {
    for element in elements {
        match element {
            Element::Text(s) => {
                // Remove newlines if they are anywhere except as the last character.
                let newlines_removed = match s.bytes().filter(|&b| b == b'\n').count() {
                    0 => Cow::Borrowed(s),
                    1 if s.ends_with('\n') => Cow::Borrowed(s),
                    _ => {
                        let mut owned = s.replace('\n', "");
                        if s.ends_with('\n') {
                            owned.push('\n');
                        }
                        Cow::Owned(owned)
                    }
                };

                let layout_fn = wrapper.unwrap_or(TextLayout::Normal);
                push_or_merge(&mut parsed.layout, layout_fn, newlines_removed.len());
                parsed.text.push_str(newlines_removed.as_ref());
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
                    let puny_code_parsed = url_punycode(text);
                    parsed.layout.push(TextLayout::Link(puny_code_parsed.len()));
                    parsed.text.push_str(&puny_code_parsed);
                    parsed.urls.push(url.to_string());
                }
            }
            Element::Escaped(c) => {
                let layout_fn = wrapper.unwrap_or(TextLayout::Normal);
                push_or_merge(&mut parsed.layout, layout_fn, 1);
                parsed.text.push(c);
            }
            Element::Paragraph => {
                push_or_merge(&mut parsed.layout, TextLayout::Normal, 1);
                parsed.text.push('\n');
            }
            Element::Code(s) => {
                parsed.layout.push(TextLayout::Code(s.len()));
                parsed.text.push_str(&s);
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
pub enum TextLayout {
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
pub fn rich_text_runs(
    theme: Theme,
    layout: &[TextLayout],
) -> impl Iterator<Item = TextRun> + use<'_> {
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
pub fn url_ranges(layouts: &[TextLayout]) -> Vec<Range<usize>> {
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
