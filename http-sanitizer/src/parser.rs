use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alpha1, char},
    combinator::map,
    multi::{many0, many1},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

fn parse_attribute(input: &str) -> IResult<&str, Attribute> {
    map(
        preceded(char(' '), separated_pair(alpha1, tag("="), parse_quote)),
        |(name, value)| Attribute { name, value },
    )(input)
}

fn parse_quote(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

fn parse_anchor_children(input: &str) -> IResult<&str, &str> {
    // take_until("</a>")(input)
    terminated(take_until("</a>"), tag("</a>"))(input)
}

fn parse_anchor(input: &str) -> IResult<&str, ParsedHtml> {
    let mut parse = delimited(tag("<a"), many0(parse_attribute), tag(">"));

    let (rest, attributes) = parse(input)?;
    let (rest, children) = parse_anchor_children(rest)?;

    Ok((
        rest,
        ParsedHtml::Link(Anchor {
            attributes,
            children,
        }),
    ))
}

fn parse_html(input: &str) -> IResult<&str, Vec<ParsedHtml>> {
    let mut parse = many1(alt((
        parse_anchor,
        // map(alt((take_until("<a"), take_until("\n"))), ParsedHtml::Text),
        map(take_until("<a"), ParsedHtml::Text),
    )));

    parse(input)
}

pub(crate) fn parse(input: &str) -> Result<Vec<ParsedHtml>, anyhow::Error> {
    let (rest, mut html) = parse_html(input).map_err(|e| anyhow::Error::msg(e.to_string()))?;

    html.push(ParsedHtml::Text(rest));
    Ok(html)
}

#[derive(Debug)]
pub(crate) struct Attribute<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub(crate) struct Anchor<'a> {
    pub attributes: Vec<Attribute<'a>>,
    pub children: &'a str,
}

#[derive(Debug)]
pub(crate) enum ParsedHtml<'a> {
    Text(&'a str),
    Link(Anchor<'a>),
}

#[cfg(test)]
mod test {
    use super::{parse, parse_anchor, parse_quote, ParsedHtml};

    #[test]
    fn parse_url() {
        let anchor = r#"<a href="http://www.google.com">Google</a><br/>"#;

        let (rest, ParsedHtml::Link(anchor)) = parse_anchor(anchor).unwrap() else {
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

        let (rest, ParsedHtml::Link(anchor)) = parse_anchor(anchor).unwrap() else {
            panic!("Wrong type");
        };

        assert!(anchor.attributes.len() == 2);
        assert_eq!(anchor.attributes[1].value, "http://www.google.com");
        assert_eq!(anchor.children, "Google");
        assert_eq!(rest, "<br/>");
    }

    #[test]
    fn parse_comment() {
        let comment = r#"This is a test with a <a href="http://www.google.com/">Google</a> Link. <a href="www.google.com">blah</a> Hello"#;

        let result = parse(comment);

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
