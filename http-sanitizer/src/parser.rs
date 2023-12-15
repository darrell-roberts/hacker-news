//! A simple html parser that targets anchor elements.
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while1, take_while_m_n},
    character::complete::{alpha1, char, space1},
    combinator::{cut, eof, map, map_opt, map_res, rest, value},
    error::{context, ContextError, FromExternalError, ParseError, VerboseError},
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    AsChar, IResult,
};
use std::num::ParseIntError;

/// An html attribute name value pair.
#[derive(Debug, Clone, Copy)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

/// An html anchor tag.
#[derive(Debug, Clone)]
pub struct Anchor<'a> {
    /// Anchor attributes.
    pub attributes: Vec<Attribute<'a>>,
    /// Child elements.
    pub children: &'a str,
}

/// A simple Html element.
#[derive(Debug, Clone)]
pub enum Element<'a> {
    /// Anything that is not a link.
    // Text(&'a str),
    Text(&'a str),
    /// A link.
    Link(Anchor<'a>),
    /// Html escaped charater
    Escaped(char),
    /// Paragraph tag.
    Paragraph,
}

fn parse_paragraph<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_paragraph",
        value(Element::Paragraph, tag_no_case("<p>")),
    )(input)
}

fn is_hex_digit(c: char) -> bool {
    c.is_hex_digit()
}

fn parse_hex<'a, E>(input: &'a str) -> IResult<&'a str, u32, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    context(
        "parse_hex",
        map_res(take_while_m_n(2, 2, is_hex_digit), |s: &str| {
            u32::from_str_radix(s, 16)
        }),
    )(input)
}

fn parse_escaped<'a, E>(input: &'a str) -> IResult<&str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let hex_parse = context(
        "escaped_tag",
        delimited(tag("&#x"), cut(parse_hex), tag(";")),
    );
    let mut parse = context(
        "parse_escaped",
        map_opt(hex_parse, |n| char::from_u32(n).map(Element::Escaped)),
    );

    parse(input)
}

fn parse_escaped_tag<'a, E>(input: &'a str) -> IResult<&str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let quote = value(Element::Escaped('\"'), tag("&quot;"));
    let gt = value(Element::Escaped('>'), tag("&gt;"));
    let lt = value(Element::Escaped('<'), tag("&lt;"));
    let ampersand = value(Element::Escaped('&'), tag("&amp;"));
    let apos = value(Element::Escaped('\''), tag("&apos;"));

    let mut parse = alt((quote, gt, lt, ampersand, apos));
    parse(input)
}

fn parse_text<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let parse = take_while1(|c| c != '<' && c != '&');
    context("parse_text", map(parse, |s: &str| Element::Text(s)))(input)
}

pub fn parse_elements<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Element>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parsers = alt((
        parse_text,
        parse_anchor,
        parse_paragraph,
        parse_escaped,
        parse_escaped_tag,
        map(rest, Element::Text),
    ));
    let mut parser = context("parse_elements", many_till(parsers, eof));

    let (rest, (mut result, _eof)) = parser(input)?;
    if !rest.is_empty() {
        result.push(Element::Text(rest));
    }
    Ok((rest, result))
}

/// Parse an html attribute name value pair.
fn parse_attribute<'a, E>(input: &'a str) -> IResult<&'a str, Attribute, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_attribute",
        map(
            preceded(space1, separated_pair(alpha1, tag("="), parse_quote)),
            |(name, value)| Attribute { name, value },
        ),
    )(input)
}

/// Parse a quoted string.
fn parse_quote<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_quote",
        delimited(char('"'), take_until("\""), char('"')),
    )(input)
}

/// Parse child elements of an anchor.
fn parse_anchor_children<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    // let escaped = alt((value("/", tag("&#x2F;")), value("\"", tag("&quot;"))));
    // let value = many0(alt((
    //     take_until("&"),
    //     take_until("</a>"),
    //     take_until("</A>"),
    //     escaped,
    // )));

    let parser = terminated(
        alt((take_until("</a>"), take_until("</A>"))),
        // value,
        alt((tag("</a>"), tag("</A>"))),
    );
    context(
        "parse_anchor_children",
        // map(parser, |ss| ss.into_iter().collect::<String>()),
        parser,
    )(input)
}

fn parse_attr<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Attribute>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_attr",
        delimited(
            // alt((tag("<a"), tag("<A"))),
            tag_no_case("<a"),
            many0(parse_attribute),
            tag(">"),
        ),
    )(input)
}

/// Parse an anchor element.
fn parse_anchor<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_anchor",
        map(
            pair(parse_attr, parse_anchor_children),
            |(attributes, children)| {
                Element::Link(Anchor {
                    attributes,
                    children,
                })
            },
        ),
    )(input)
}

// fn parse_element(input: &[u8]) -> IResult<&[u8], Element> {
//     alt((parse_escaped, take_until("<a"), take_while1(|_| true)))(input)
// }

/// Parse an html string.
pub(crate) fn parse_html(input: &str) -> anyhow::Result<Vec<Element>> {
    many1(alt((
        parse_anchor::<VerboseError<&str>>,
        map(
            alt((take_until("<a"), take_while1(|_| true))),
            |bs: &str| Element::Text(bs),
        ),
    )))(input)
    .map_err(|e| anyhow::Error::msg(e.to_string()))
    .map(|(_, html)| html)
}

#[cfg(test)]
mod test {
    use super::{parse_anchor, parse_elements, parse_escaped, parse_html, parse_quote, Element};
    use crate::parser::parse_paragraph;
    use nom::{
        error::{convert_error, VerboseError},
        Err,
    };

    #[test]
    fn parse_url() {
        let anchor = r#"<a href="http://www.google.com">Google</a><br/>"#;

        let (rest, Element::Link(anchor)) = parse_anchor::<VerboseError<&str>>(anchor).unwrap()
        else {
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

        let (rest, Element::Link(anchor)) = parse_anchor::<VerboseError<&str>>(anchor).unwrap()
        else {
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

        let (rest, Element::Link(anchor)) = parse_anchor::<VerboseError<&str>>(anchor).unwrap()
        else {
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

        let (rest, v) = parse_quote::<VerboseError<&str>>(q).unwrap();

        assert_eq!(v, "hello");
        assert!(rest.is_empty());
    }

    #[test]
    fn test_escaped_slash() {
        let s = "&#x2F;some more stuff";

        let (rest, el) = parse_escaped::<VerboseError<&str>>(s).unwrap();

        assert!(matches!(el, Element::Escaped('/')));
        assert_eq!(rest, "some more stuff");
    }

    #[test]
    fn test_parse_paragraph() {
        let s = "<P>some more stuff";

        let (rest, el) = parse_paragraph::<VerboseError<&str>>(s).unwrap();

        assert!(matches!(el, Element::Paragraph));
        assert_eq!(rest, "some more stuff");
    }

    #[test]
    fn test_elements() {
        let s = r#"123h&#x2F; <P>&#x2F;&#x23;<P>Hello<P>
            <a href="some url">some link</a>"#;

        let el = parse_elements::<VerboseError<&str>>(s);

        match el {
            Ok(elements) => {
                dbg!(&elements);
            }
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                eprintln!("error: {}", convert_error(s, err));
                panic!("failed");
            }
            Err(err) => {
                dbg!(&err);
                panic!("failed");
            }
        }
    }
}
