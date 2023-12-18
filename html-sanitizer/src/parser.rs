//! A simple html parser that targets anchor elements.
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while1, take_while_m_n},
    character::complete::{alpha1, anychar, char, space1},
    combinator::{cut, eof, map, map_opt, map_res, rest, value},
    error::{context, ContextError, FromExternalError, ParseError, VerboseError},
    multi::{many0, many1, many_till},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    AsChar, IResult, Parser,
};
use std::num::ParseIntError;

/// An html attribute name value pair.
#[derive(Debug, Clone)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: String,
}

/// An html anchor tag.
#[derive(Debug, Clone)]
pub struct Anchor<'a> {
    /// Anchor attributes.
    pub attributes: Vec<Attribute<'a>>,
    /// Child elements.
    pub children: String,
}

/// A simple Html element.
#[derive(Debug, Clone)]
pub enum Element<'a> {
    /// Regular text.
    Text(&'a str),
    /// A link.
    Link(Anchor<'a>),
    /// Html escaped charater
    Escaped(char),
    /// Paragraph tag.
    Paragraph,
    /// Source code block.
    Code(String),
    /// Italic text.
    Italic(String),
    /// Bold text.
    Bold(String),
}

fn parse_bold<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse = delimited(
        tag("<b>"),
        take_until("</b>").and_then(parse_escaped_text),
        tag("</b>"),
    );
    context("parse_bold", map(parse, Element::Bold))(input)
}

fn parse_italic<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parse = delimited(
        tag("<i>"),
        take_until("</i>").and_then(parse_escaped_text),
        tag("</i>"),
    );
    context("parse_italic", map(parse, Element::Italic))(input)
}

fn parse_escaped_text<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(
        many0(alt((parse_escaped_character, parse_escaped_tag, anychar))),
        |v| v.into_iter().collect(),
    )(input)
}

fn parse_code<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
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

fn parse_escaped_character<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let hex_parse = context(
        "escaped_tag",
        delimited(tag("&#x"), cut(parse_hex), tag(";")),
    );
    let mut parse = context("parse_escaped", map_opt(hex_parse, char::from_u32));

    parse(input)
}

fn parse_escaped<'a, E>(input: &'a str) -> IResult<&str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    map(parse_escaped_character, Element::Escaped)(input)
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

    alt((quote, gt, lt, ampersand, apos))(input)
}

fn parse_escaped_tag_into_element<'a, E>(input: &'a str) -> IResult<&str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(parse_escaped_tag, Element::Escaped)(input)
}

fn parse_text<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    let parse = take_while1(|c| c != '<' && c != '&');
    context("parse_text", map(parse, |s: &str| Element::Text(s)))(input)
}

/// Parse an html attribute name value pair.
fn parse_attribute<'a, E>(input: &'a str) -> IResult<&'a str, Attribute, E>
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
fn parse_anchor_children<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parser = terminated(
        alt((take_until("</a>"), take_until("</A>"))).and_then(parse_escaped_text),
        alt((tag("</a>"), tag("</A>"))),
    );
    context("parse_anchor_children", parser)(input)
}

fn parse_attr<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Attribute>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    context(
        "parse_attr",
        delimited(tag_no_case("<a"), many0(parse_attribute), tag(">")),
    )(input)
}

/// Parse an anchor element.
fn parse_anchor<'a, E>(input: &'a str) -> IResult<&'a str, Element, E>
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
    )(input)
}

/// Parse html encoded string into a logical [`Element`]s.
pub fn parse_elements<'a, E>(input: &'a str) -> IResult<&'a str, Vec<Element>, E>
where
    E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>,
{
    let parsers = alt((
        parse_text,
        parse_anchor,
        parse_paragraph,
        parse_escaped,
        parse_escaped_tag_into_element,
        parse_code,
        parse_italic,
        parse_bold,
        map(rest, Element::Text),
    ));
    let mut parser = context("parse_elements", many_till(parsers, eof));

    let (rest, (mut result, _eof)) = parser(input)?;
    if !rest.is_empty() {
        result.push(Element::Text(rest));
    }
    Ok((rest, result))
}

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
    use super::{
        parse_anchor, parse_code, parse_elements, parse_escaped, parse_html, parse_paragraph,
        parse_quote, Element,
    };
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
            Ok((rest, elements)) => {
                assert!(rest.is_empty());
                assert!(!elements.is_empty());
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

    #[test]
    fn test_code() {
        let s = r#"<pre><code>
            if x {
                y = 0
            }
        </code></pre>"#;

        let el = parse_code::<VerboseError<&str>>(s);
        match el {
            Ok((rest, element)) => {
                assert!(rest.is_empty());
                assert!(matches!(element, Element::Code(_)));
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

    #[test]
    fn testy() {
        let s = "I really, really loathe it when scientists advocate for something (in this case planting tons of trees) and then get faux shocked when people use that information to their economic benefit (&quot;I didn&#x27;t mean plant trees <i>and</i> still burn fossil fuels!&quot;)<p>A good analogy to me is the &quot;anti-fat&quot; nutrition crowd in the 90s (remember &quot;the food pyramid&quot; anyone??) I was reading an article about this whole debacle a while back, and remember one scientist lamenting &quot;The advice on its own was good advice, but we never imagined the rise of Snack Wells.&quot; If anyone doesn&#x27;t know, Snack Wells were a cookie brand in the 90s that were fat-free but loaded with sugar. They had the effect of getting you just as fat (they had a ton of calories), with probably a higher risk of type 2 diabetes, but with no fat they left you feeling hungry and they tasted a bit like cardboard.<p>But the scientist&#x27;s defense was utter baloney. Of course if you convince the populace that fat is evil and you can avoid weight gain just by avoiding dietary fat that food companies will respond accordingly.<p>The same thing applies here. It&#x27;s ridiculous for a scientist to think that his report about how planting lots of trees can counteract fossil fuel emissions wouldn&#x27;t be latched on to by fossil fuel companies to say they &quot;offset&quot; their new emissions by planting more trees.";

        let result = parse_elements::<VerboseError<&str>>(s);

        dbg!(&result);
    }
}
