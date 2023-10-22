use parser::{parse, ParsedHtml};
use std::borrow::Cow;

mod parser;

/// Transform any html anchor links inside a comment.
pub fn sanitize_html<'a>(input: &'a str) -> Cow<'a, str> {
    let Ok(elements) = parse(input) else {
        return Cow::Borrowed(input);
    };

    if elements.iter().all(|el| matches!(el, ParsedHtml::Text(_))) {
        return Cow::Borrowed(input);
    }

    let modified = elements.into_iter().fold(String::new(), |mut s, elem| {
        match elem {
            ParsedHtml::Text(t) => {
                s.push_str(t);
            }
            ParsedHtml::Link(l) => {
                if let Some(att) = l.attributes.iter().find(|a| a.name == "href") {
                    s.push_str(att.value);
                }
            }
        }
        s
    });

    Cow::Owned(modified)
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

        let transformed = sanitize_html(&test);

        let expected = r#"
            Hello this is a comment.
            I have a http://www.google.com/ link.
            Bye.
        "#;

        assert_eq!(transformed, expected);
    }
}
