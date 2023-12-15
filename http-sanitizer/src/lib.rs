use log::error;
use nom::error::VerboseError;
use parser::parse_html;
use std::borrow::Cow;

mod parser;

pub use parser::{parse_elements, Element};

pub fn convert_html(input: &str) -> Cow<str> {
    match parse_elements::<VerboseError<&str>>(input) {
        Ok((_, elements)) => {
            let mut result = String::new();
            for element in elements {
                match element {
                    Element::Text(s) => result.push_str(s),
                    Element::Link(l) => {
                        if let Some(att) = l.attributes.iter().find(|a| a.name == "href") {
                            // TODO: Should be in parser.
                            result.push_str(&att.value.replace("&#x2F;", "/"));

                            if l.children != att.value && !l.children.starts_with("http") {
                                result.push('(');
                                result.push_str(l.children);
                                result.push(')');
                            }
                        }
                    }
                    Element::Escaped(c) => result.push(c),
                    Element::Paragraph => result.push_str("\n\n"),
                }
            }
            Cow::Owned(result)
        }
        Err(err) => {
            error!("Parsing failed: {err}");
            Cow::Borrowed(input)
        }
    }
}

/// Transform any html anchor links inside a comment.
pub fn sanitize_html(input: String) -> String {
    let elements = match parse_html(&input) {
        Ok(el) => el,
        Err(err) => {
            error!("Failed to parse input: {err}");
            return input;
        }
    };

    // All Text does not require any sanitization.
    if elements.iter().all(|el| matches!(el, Element::Text(_))) {
        return input;
    }

    elements.into_iter().fold(String::new(), |mut s, elem| {
        match elem {
            Element::Text(t) => {
                // let str: &str = std::str::from_utf8(t).unwrap_or_default();
                // s.push_str(str);
                s.push_str(t);
            }
            Element::Link(l) => {
                if let Some(att) = l.attributes.iter().find(|a| a.name == "href") {
                    s.push_str(att.value);

                    if l.children != att.value && !l.children.starts_with("http") {
                        s.push('(');
                        s.push_str(l.children);
                        s.push(')');
                    }
                }
            }
            Element::Paragraph => {
                s.push('\n');
            }
            Element::Escaped(c) => {
                s.push(c);
            }
        }
        s
    })
}

#[cfg(test)]
mod test {
    use super::sanitize_html;

    #[test]
    fn test_transform() {
        let test = r#"
            Hello this is a comment.
            I have a <a href="http://www.google.com/">Google</a> link.
            Bye.
        "#;

        let transformed = sanitize_html(String::from(test));

        let expected = r#"
            Hello this is a comment.
            I have a http://www.google.com/(Google) link.
            Bye.
        "#;

        assert_eq!(transformed, expected);
    }

    #[test]
    fn test_transform_no_link() {
        let comment = r#"
            I am a comment. I have no
            links.
            Bye.
        "#;

        let transformed = sanitize_html(String::from(comment));

        assert_eq!(transformed, comment);
    }
}
