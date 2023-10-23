///! A simple html parser that targets anchor elements.
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{alpha1, char, space1},
    combinator::map,
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    IResult,
};

/// An html attribute name value pair.
#[derive(Debug)]
pub(crate) struct Attribute<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

/// An html anchor tag.
#[derive(Debug)]
pub(crate) struct Anchor<'a> {
    /// Anchor attributes.
    pub attributes: Vec<Attribute<'a>>,
    /// Child elements.
    pub children: &'a str,
}

/// A simple Html element.
#[derive(Debug)]
pub(crate) enum Element<'a> {
    /// Anything that is not a link.
    Text(&'a str),
    /// A link.
    Link(Anchor<'a>),
}

/// Parse an html attribute name value pair.
fn parse_attribute(input: &str) -> IResult<&str, Attribute> {
    map(
        preceded(space1, separated_pair(alpha1, tag("="), parse_quote)),
        |(name, value)| Attribute { name, value },
    )(input)
}

/// Parse a quoted string.
fn parse_quote(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

/// Parse child elements of an anchor.
fn parse_anchor_children(input: &str) -> IResult<&str, &str> {
    terminated(
        alt((take_until("</a>"), take_until("</A>"))),
        alt((tag("</a>"), tag("</A>"))),
    )(input)
}

/// Parse an anchor element.
fn parse_anchor(input: &str) -> IResult<&str, Element> {
    let parse_attr = delimited(
        alt((tag("<a"), tag("<A"))),
        many0(parse_attribute),
        tag(">"),
    );

    map(
        pair(parse_attr, parse_anchor_children),
        |(attributes, children)| {
            Element::Link(Anchor {
                attributes,
                children,
            })
        },
    )(input)
}

/// Parse an html string.
pub(crate) fn parse_html(input: &str) -> anyhow::Result<Vec<Element>> {
    many1(alt((
        parse_anchor,
        map(
            alt((take_until("<a"), take_while1(|_| true))),
            Element::Text,
        ),
    )))(input)
    .map_err(|e| anyhow::Error::msg(e.to_string()))
    .map(|(_, html)| html)
}

#[cfg(test)]
mod test {
    use super::{parse_anchor, parse_html, parse_quote, Element};

    #[test]
    fn parse_url() {
        let anchor = r#"<a href="http://www.google.com">Google</a><br/>"#;

        let (rest, Element::Link(anchor)) = parse_anchor(anchor).unwrap() else {
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

        let (rest, Element::Link(anchor)) = parse_anchor(anchor).unwrap() else {
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

        let (rest, Element::Link(anchor)) = parse_anchor(anchor).unwrap() else {
            panic!("Wrong type");
        };

        assert!(anchor.attributes.len() == 2);
        assert_eq!(anchor.attributes[1].value, "http://www.google.com");
        assert_eq!(anchor.children, "Google");
        assert_eq!(rest, "<br/>");
    }

    #[test]
    fn parse_comment() {
        let comment = r#"
            This is a test with a <a href="http://www.google.com/">Google</a> Link. <a href="www.google.com">blah</a> Hello
            "#;
        let result = parse_html(comment);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_comment_no_link() {
        let comment = r#"
            I am a comment. I have no
            links.
            Bye.
        "#;

        let result = parse_html(comment);
        assert!(result.is_ok());
    }

    #[test]
    fn quote() {
        let q = r#""hello""#;

        let (rest, v) = parse_quote(q).unwrap();

        assert_eq!(v, "hello");
        assert!(rest.is_empty());
    }
}
