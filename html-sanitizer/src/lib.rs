use log::error;
use nom::error::VerboseError;
use parser::parse_html;

mod parser;

pub use parser::{parse_elements, Element};

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
                s.push_str(t);
            }
            Element::Link(l) => {
                if let Some(att) = l.attributes.iter().find(|a| a.name == "href") {
                    s.push_str(&att.value);

                    if l.children != att.value && !l.children.starts_with("http") {
                        s.push('(');
                        s.push_str(&l.children);
                        s.push(')');
                    }
                }
            }
            Element::Paragraph => {
                s.push_str("\n\n");
            }
            Element::Escaped(c) => {
                s.push(c);
            }
            Element::Code(code) => {
                s.push_str("------begin code-----");
                s.push_str(&code);
                s.push_str("----end code---------");
            }
            Element::Italic(i) => {
                s.push_str(&i);
            }
            Element::Bold(b) => s.push_str(&b),
        }
        s
    })
}

/// Parse the input str into elements.
pub fn as_elements(input: &str) -> Vec<Element> {
    parse_elements::<VerboseError<&str>>(input)
        .map(|(_, v)| v)
        .unwrap_or_else(|err| {
            error!("Failed to parse input: {err}");
            vec![Element::Text(input)]
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
