use log::error;
use nom::error::VerboseError;

mod parser;

pub use parser::{Anchor, Element};

/// Parse the input str into elements.
pub fn parse_elements(input: &str) -> Vec<Element> {
    parser::parse_nodes::<VerboseError<&str>>(input)
        .map(|(_, v)| v)
        .unwrap_or_else(|err| {
            error!("Failed to parse input: {err}");
            vec![Element::Text(input)]
        })
}
