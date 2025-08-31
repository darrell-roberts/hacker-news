//! A simple html parser that targets anchor elements.
use crate::{Anchor, Attribute, Element};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while1, take_while_m_n},
    character::complete::{alpha1, anychar, char, space1},
    combinator::{cut, map, map_opt, map_res, value},
    error::context,
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    AsChar, IResult, Parser,
};

#[cfg(test)]
mod parser_tests;

/// A parse result with a &str input.
type ParseResult<'a, O> = IResult<&'a str, O>;

pub fn parse_nodes(input: &str) -> ParseResult<'_, Vec<Element<'_>>> {
    many0(alt((parse_tag, parse_text))).parse(input)
}

fn parse_tag<'a>(input: &'a str) -> ParseResult<'a, Element<'a>> {
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

fn parse_bold(input: &str) -> ParseResult<'_, Element<'_>> {
    let bold = delimited(tag("<b>"), parse_nodes, tag("</b>"));
    context("parse_bold", map(bold, Element::Bold)).parse(input)
}

fn parse_italic(input: &str) -> ParseResult<'_, Element<'_>> {
    let italic = delimited(tag("<i>"), parse_nodes, tag("</i>"));
    context("parse_italic", map(italic, Element::Italic)).parse(input)
}

fn parse_escaped_text(input: &str) -> ParseResult<'_, String> {
    map(
        many0(alt((parse_escaped_character, parse_escaped_tag, anychar))),
        |v| v.into_iter().collect(),
    )
    .parse(input)
}

fn parse_code(input: &str) -> ParseResult<'_, Element<'_>> {
    let code = delimited(
        tag_no_case("<pre><code>"),
        take_until("</code></pre>").and_then(parse_escaped_text),
        tag_no_case("</code></pre>"),
    );

    map(code, Element::Code).parse(input)
}

fn parse_paragraph(input: &str) -> ParseResult<'_, Element<'_>> {
    context(
        "parse_paragraph",
        value(Element::Paragraph, tag_no_case("<p>")),
    )
    .parse(input)
}

fn is_hex_digit(c: char) -> bool {
    c.is_hex_digit()
}

fn parse_hex(input: &str) -> ParseResult<'_, u32> {
    context(
        "parse_hex",
        map_res(take_while_m_n(2, 2, is_hex_digit), |s: &str| {
            u32::from_str_radix(s, 16)
        }),
    )
    .parse(input)
}

fn parse_escaped_character(input: &str) -> ParseResult<'_, char> {
    let hex_parse = context(
        "escaped_tag",
        delimited(tag("&#x"), cut(parse_hex), tag(";")),
    );
    context("parse_escaped", map_opt(hex_parse, char::from_u32)).parse(input)
}

fn parse_escaped(input: &str) -> ParseResult<'_, Element<'_>> {
    map(
        alt((parse_escaped_character, parse_escaped_tag)),
        Element::Escaped,
    )
    .parse(input)
}

fn parse_escaped_tag(input: &str) -> ParseResult<'_, char> {
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

fn parse_text(input: &str) -> ParseResult<'_, Element<'_>> {
    let text = take_while1(|c| c != '<' && c != '&');
    context("parse_text", map(text, |s: &str| Element::Text(s))).parse(input)
}

/// Parse an html attribute name value pair.
fn parse_attribute(input: &str) -> ParseResult<'_, Attribute<'_>> {
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
fn parse_quote(input: &str) -> ParseResult<'_, &str> {
    context(
        "parse_quote",
        delimited(char('"'), take_until("\""), char('"')),
    )
    .parse(input)
}

/// Parse child elements of an anchor.
fn parse_anchor_children(input: &str) -> ParseResult<'_, String> {
    let anchor = terminated(
        alt((take_until("</a>"), take_until("</A>"))).and_then(parse_escaped_text),
        alt((tag("</a>"), tag("</A>"))),
    );
    context("parse_anchor_children", anchor).parse(input)
}

fn parse_attr(input: &str) -> ParseResult<'_, Vec<Attribute<'_>>> {
    context(
        "parse_attr",
        delimited(tag_no_case("<a"), many0(parse_attribute), tag(">")),
    )
    .parse(input)
}

/// Parse an anchor element.
fn parse_anchor(input: &str) -> ParseResult<'_, Element<'_>> {
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
