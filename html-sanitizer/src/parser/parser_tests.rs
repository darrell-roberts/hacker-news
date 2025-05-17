use super::{
    parse_anchor, parse_code, parse_escaped, parse_nodes, parse_paragraph, parse_quote, Element,
};
use cool_asserts::assert_matches;
use nom::{error::Error, Err};

#[test]
fn parse_url() {
    let anchor = r#"<a href="http://www.google.com">Google</a><br/>"#;

    let (rest, Element::Link(anchor)) = parse_anchor::<Error<&str>>(anchor).unwrap() else {
        panic!("Wrong type");
    };

    assert!(anchor.attributes.len() == 1);
    assert_eq!(anchor.attributes[0].value, "http://www.google.com");
    assert_eq!(anchor.children, "Google");
    assert_eq!(rest, "<br/>");
}

#[test]
fn parse_url_upper() {
    let anchor = r#"<A href="http://www.google.com">Google</A><br/>"#;

    let (rest, Element::Link(anchor)) = parse_anchor::<Error<&str>>(anchor).unwrap() else {
        panic!("Wrong type");
    };

    assert!(anchor.attributes.len() == 1);
    assert_eq!(anchor.attributes[0].value, "http://www.google.com");
    assert_eq!(anchor.children, "Google");
    assert_eq!(rest, "<br/>");
}

#[test]
fn parse_alt_url() {
    let anchor = r#"<a target="_blank" href="http://www.google.com">Google</a><br/>"#;

    let (rest, Element::Link(anchor)) = parse_anchor::<Error<&str>>(anchor).unwrap() else {
        panic!("Wrong type");
    };

    assert!(anchor.attributes.len() == 2);
    assert_eq!(anchor.attributes[1].value, "http://www.google.com");
    assert_eq!(anchor.children, "Google");
    assert_eq!(rest, "<br/>");
}

#[test]
fn quote() {
    let q = r#""hello""#;

    let (rest, v) = parse_quote::<Error<&str>>(q).unwrap();

    assert_eq!(v, "hello");
    assert!(rest.is_empty());
}

#[test]
fn test_escaped_slash() {
    let s = "&#x2F;some more stuff";

    let (rest, el) = parse_escaped::<Error<&str>>(s).unwrap();

    assert!(matches!(el, Element::Escaped('/')));
    assert_eq!(rest, "some more stuff");
}

#[test]
fn test_parse_paragraph() {
    let s = "<P>some more stuff";

    let (rest, el) = parse_paragraph::<Error<&str>>(s).unwrap();

    assert!(matches!(el, Element::Paragraph));
    assert_eq!(rest, "some more stuff");
}

#[test]
fn test_elements() {
    let s = r#"123h&#x2F; <P>&#x2F;&#x23;<P>Hello<P>
            <a href="some url">some link</a>"#;

    let el = parse_nodes::<Error<&str>>(s);

    match el {
        Ok((rest, elements)) => {
            assert!(rest.is_empty());
            assert!(!elements.is_empty());
            dbg!(&elements);
        }
        Err(Err::Error(err)) | Err(Err::Failure(err)) => {
            eprintln!("error: {err}");
            panic!("failed");
        }
        Err(err) => {
            dbg!(&err);
            panic!("failed");
        }
    }
}

#[test]
fn test_code() {
    let s = r#"<pre><code>
            if x {
                y = 0
            }
        </code></pre>"#;

    let el = parse_code::<Error<&str>>(s);
    match el {
        Ok((rest, element)) => {
            assert!(rest.is_empty());
            assert!(matches!(element, Element::Code(_)));
        }
        Err(Err::Error(err)) | Err(Err::Failure(err)) => {
            eprintln!("error: {err}");
            panic!("failed");
        }
        Err(err) => {
            dbg!(&err);
            panic!("failed");
        }
    }
}

#[test]
fn test_nested() {
    let s = r#"<b>This bold <i>italic&reg;</i>.</b>And some Code<pre><code>println!("")</code></pre> and more text"#;
    let (rest, nodes) = parse_nodes::<Error<&str>>(s).unwrap();

    assert_eq!(rest, "");
    assert_matches!(
        nodes,
        [
            Element::Bold(inner) => {
                assert_matches!(inner, [
                    Element::Text("This bold "),
                    Element::Italic(italic) => {
                        assert_matches!(italic,
                            [Element::Text("italic"), Element::Escaped('®')]
                        )
                    },
                    Element::Text("."),
                ])
            },
            Element::Text("And some Code"),
            Element::Code(code) => {
                assert_eq!(code,"println!(\"\")");
            },
            Element::Text(" and more text"),
        ],
    );
}
