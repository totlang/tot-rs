use nom::{
    branch::alt,
    bytes::complete::{is_not, take_while_m_n},
    character::complete::{char, multispace1},
    combinator::{cut, map, map_opt, map_res, value, verify},
    multi::fold_many0,
    sequence::{delimited, preceded},
};

use super::PResult;

fn parse_unicode(i: &str) -> PResult<char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

    map_opt(parse_u32, |value| std::char::from_u32(value))(i)
}

fn parse_escaped_char(i: &str) -> PResult<char> {
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(i)
}

fn parse_escaped_whitespace(i: &str) -> PResult<&str> {
    preceded(char('\\'), multispace1)(i)
}

fn literal(i: &str) -> PResult<&str> {
    let not_quote_slash = is_not("\"\\");

    verify(not_quote_slash, |s: &str| !s.is_empty())(i)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWhitespace,
}

fn parse_fragment<'a>(i: &'a str) -> PResult<StringFragment<'a>> {
    alt((
        map(literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWhitespace, parse_escaped_whitespace),
    ))(i)
}

pub fn parse_string(i: &str) -> PResult<String> {
    let build_string = fold_many0(parse_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(v) => string.push_str(v),
            StringFragment::EscapedChar(v) => string.push(v),
            StringFragment::EscapedWhitespace => {}
        }
        string
    });

    delimited(char('"'), cut(build_string), char('"'))(i)
}

#[cfg(test)]
mod test {
    #[test]
    fn parse_unicode() {
        let (_, par) = super::parse_unicode("u{1F602}").unwrap();

        assert_eq!(par, '😂');
    }

    #[test]
    fn parse_escaped_char() {
        let (_, par) = super::parse_escaped_char("\\n").unwrap();
        assert_eq!(par, '\n');

        let (_, par) = super::parse_escaped_char("\\\"").unwrap();
        assert_eq!(par, '\"');
    }

    #[test]
    fn parse_escaped_whitespace() {
        let (_, par) = super::parse_escaped_whitespace("\\ ").unwrap();
        assert_eq!(par, " ");

        let (_, par) = super::parse_escaped_whitespace("\\     ").unwrap();
        assert_eq!(par, "     ");
    }

    #[test]
    fn parse_literal() {
        let (_, par) = super::literal("hello world").unwrap();

        assert!(!par.is_empty());
        assert_eq!(par, "hello world");
    }

    #[test]
    fn parse_fragment() {
        let (_, par) = super::parse_fragment("hello world").unwrap();

        assert_eq!(par, super::StringFragment::Literal("hello world"));
        if let super::StringFragment::Literal(v) = par {
            assert_eq!(v, "hello world");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn parse_string() {
        let (_, par) = super::parse_string("\"this is a test\"").unwrap();
        assert_eq!(par, "this is a test");

        let (_, par) = super::parse_string("\"\n\n\n\n\"").unwrap();
        assert_eq!(par, "\n\n\n\n");

        let (_, par) = super::parse_string("\"     \"").unwrap();
        assert_eq!(par, "     ");
    }
}
