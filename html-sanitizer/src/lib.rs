//! Simple html parser for the following element types:
//!
//! - `<b>` bold
//! - `<i>` italic
//! - `<p>` paragraph
//! - `<a>` anchor
//! - `<pre><code></code></pre>` monospaced code
use std::borrow::Cow;

use log::{error, warn};

mod parser;

/// An html attribute name value pair.
#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: String,
}

/// An html anchor tag.
#[derive(Debug, Clone)]
pub struct Anchor<'a> {
    /// Anchor attributes.
    pub attributes: Vec<Attribute<'a>>,
    /// Child elements.
    pub children: String,
}

/// A simple Html element.
#[derive(Debug, Clone)]
pub enum Element<'a> {
    /// Regular text.
    Text(&'a str),
    /// A link.
    Link(Anchor<'a>),
    /// Html escaped character
    Escaped(char),
    /// Paragraph tag.
    Paragraph,
    /// Source code block.
    Code(String),
    /// Italic text.
    Italic(Vec<Element<'a>>),
    /// Bold text.
    Bold(Vec<Element<'a>>),
}

/// Parse the input str into elements.
pub fn parse_elements(input: &str) -> Vec<Element<'_>> {
    parser::parse_nodes(input)
        .inspect(|(rest, _)| {
            if !rest.is_empty() {
                warn!("Unparsed text left over: \"{rest}\"")
            }
        })
        .map(|(_, v)| v)
        .unwrap_or_else(|err| {
            error!("Failed to parse input: {err}");
            Vec::from(&[Element::Text(input)])
        })
}

const ESCAPED_CHAR_MAP: &[(char, &str)] = &[
    ('&', "&amp;"),
    ('<', "&lt;"),
    ('>', "&gt;"),
    ('"', "&quot;"),
    ('\'', "&apos;"),
    ('©', "&copy;"),
    ('®', "&reg;"),
    ('™', "&trade;"),
    ('£', "&pound;"),
    ('€', "&euro;"),
    ('°', "&deg;"),
];

pub fn escape_html<'a>(input: &'a str) -> Cow<'a, str> {
    if input
        .chars()
        .any(|c| ESCAPED_CHAR_MAP.iter().any(|map| map.0 == c))
    {
        let mut modified_buffer = String::new();
        for c in input.chars() {
            if let Some(replacement) = ESCAPED_CHAR_MAP.iter().find(|map| map.0 == c) {
                modified_buffer.push_str(replacement.1);
            } else {
                modified_buffer.push(c);
            }
        }

        modified_buffer.into()
    } else {
        input.into()
    }
}
