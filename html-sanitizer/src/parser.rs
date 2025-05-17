//! A simple html parser that targets anchor elements.
use crate::{Anchor, Attribute, Element};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while1, take_while_m_n},
    character::complete::{alpha1, anychar, char, space1},
    combinator::{cut, map, map_opt, map_res, value},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    AsChar, IResult, Parser,
};
use std::num::ParseIntError;

pub fn parse_nodes<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Element<'a>>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    many0(alt((parse_tag, parse_text))).parse(input)
}

fn parse_tag<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    alt((
        parse_bold,
        parse_italic,
        parse_anchor,
        parse_paragraph,
        parse_code,
        parse_escaped,
    ))
    .parse(input)
}

fn parse_bold<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse = delimited(tag("<b>"), parse_nodes, tag("</b>"));
    context("parse_bold", map(parse, Element::Bold)).parse(input)
}

fn parse_italic<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse = delimited(tag("<i>"), parse_nodes, tag("</i>"));
    context("parse_italic", map(parse, Element::Italic)).parse(input)
}

fn parse_escaped_text<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(
        many0(alt((parse_escaped_character, parse_escaped_tag, anychar))),
        |v| v.into_iter().collect(),
    )
    .parse(input)
}

fn parse_code<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse = delimited(
        tag_no_case("<pre><code>"),
        take_until("</code></pre>").and_then(parse_escaped_text),
        tag_no_case("</code></pre>"),
    );

    map(parse, Element::Code).parse(input)
}

fn parse_paragraph<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_paragraph",
        value(Element::Paragraph, tag_no_case("<p>")),
    )
    .parse(input)
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
    )
    .parse(input)
}

fn parse_escaped_character<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let hex_parse = context(
        "escaped_tag",
        delimited(tag("&#x"), cut(parse_hex), tag(";")),
    );
    context("parse_escaped", map_opt(hex_parse, char::from_u32)).parse(input)
}

fn parse_escaped<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(
        alt((parse_escaped_character, parse_escaped_tag)),
        Element::Escaped,
    )
    .parse(input)
}

fn parse_escaped_tag<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let quote = value('\"', tag("&quot;"));
    let gt = value('>', tag("&gt;"));
    let lt = value('<', tag("&lt;"));
    let ampersand = value('&', tag("&amp;"));
    let apos = value('\'', tag("&apos;"));
    let copy = value('©', tag("&copy;"));
    let reg = value('®', tag("&reg;"));
    let trade = value('™', tag("&trade;"));
    let deg = value('°', tag("&deg;"));
    let euro = value('€', tag("&euro;"));

    alt((quote, gt, lt, ampersand, apos, copy, reg, trade, deg, euro)).parse(input)
}

fn parse_text<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let parse = take_while1(|c| c != '<' && c != '&');
    context("parse_text", map(parse, |s: &str| Element::Text(s))).parse(input)
}

/// Parse an html attribute name value pair.
fn parse_attribute<'a, E>(input: &'a str) -> IResult<&'a str, Attribute<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    context(
        "parse_attribute",
        map(
            preceded(
                space1,
                separated_pair(alpha1, tag("="), parse_quote.and_then(parse_escaped_text)),
            ),
            |(name, value)| Attribute { name, value },
        ),
    )
    .parse(input)
}

/// Parse a quoted string.
fn parse_quote<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    context(
        "parse_quote",
        delimited(char('"'), take_until("\""), char('"')),
    )
    .parse(input)
}

/// Parse child elements of an anchor.
fn parse_anchor_children<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parser = terminated(
        alt((take_until("</a>"), take_until("</A>"))).and_then(parse_escaped_text),
        alt((tag("</a>"), tag("</A>"))),
    );
    context("parse_anchor_children", parser).parse(input)
}

fn parse_attr<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Attribute<'a>>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    context(
        "parse_attr",
        delimited(tag_no_case("<a"), many0(parse_attribute), tag(">")),
    )
    .parse(input)
}

/// Parse an anchor element.
fn parse_anchor<'a, E>(input: &'a str) -> IResult<&'a str, Element<'a>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
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
    )
    .parse(input)
}

#[cfg(test)]
mod test {
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
}
