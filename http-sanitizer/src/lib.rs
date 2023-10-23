use log::error;
use parser::{parse, ParsedHtml};

mod parser;

/// Transform any html anchor links inside a comment.
pub fn sanitize_html<'a>(input: String) -> String {
    let elements = match parse(&input) {
        Ok(el) => el,
        Err(err) => {
            error!("Failed to parse input: {err}");
            return input;
        }
    };

    // All Text does not require any sanitization.
    if elements.iter().all(|el| matches!(el, ParsedHtml::Text(_))) {
        return input;
    }

    let modified = elements.into_iter().fold(String::new(), |mut s, elem| {
        match elem {
            ParsedHtml::Text(t) => {
                s.push_str(t);
            }
            ParsedHtml::Link(l) => {
                if let Some(att) = l.attributes.iter().find(|a| a.name == "href") {
                    s.push_str(att.value);

                    if l.children != att.value && !l.children.starts_with("http") {
                        s.push('(');
                        s.push_str(l.children);
                        s.push(')');
                    }
                }
            }
        }
        s
    });

    modified
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
