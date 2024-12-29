//! Simple html parser for the following element types:
//! - `<b>` bold
//! - `<i>` italic
//! - `<p>` paragraph
//! - `<a>` anchor
//! - `<pre><code>` monospaced code
use log::{error, warn};
use nom::error::VerboseError;

mod parser;

pub use parser::{Anchor, Element};

/// Parse the input str into elements.
pub fn parse_elements(input: &str) -> Vec<Element> {
    parser::parse_nodes::<VerboseError<&str>>(input)
        .inspect(|(rest, _)| {
            if !rest.is_empty() {
                warn!("Unparsed text left over: \"{rest}\"")
            }
        })
        .map(|(_, v)| v)
        .unwrap_or_else(|err| {
            error!("Failed to parse input: {err}");
            vec![Element::Text(input)]
        })
}
