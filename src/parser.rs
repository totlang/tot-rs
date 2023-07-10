mod string;

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{
        escaped, is_not, tag, tag_no_case, take_till1, take_until, take_while, take_while_m_n,
    },
    character::complete::{
        alphanumeric0, alphanumeric1, anychar, char, digit1, i32, i64, line_ending, multispace0,
        multispace1, none_of, one_of, space0, space1,
    },
    combinator::{
        cut, eof, map, map_opt, map_res, not, opt, peek, recognize, success, value, verify,
    },
    error::{context, ParseError},
    multi::{many0, many_till, separated_list0},
    number::complete::double as nom_double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Compare, IResult, InputLength, InputTake, InputTakeAtPosition,
};

use string::parse_string as string;

#[derive(thiserror::Error, Debug)]
pub enum TotError {
    #[error("error ocurred while parsing")]
    ParseError,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TotValue {
    Void,
    Boolean(bool),
    String(String),
    Double(f64),
    List(Vec<TotValue>),
    Dict(HashMap<String, TotValue>),
}

pub type PResult<'a, T> = IResult<&'a str, T>;

fn id(i: &str) -> PResult<&str> {
    take_until(" ")(i)
}

fn key(i: &str) -> PResult<String> {
    alt((string, map(id, String::from)))(i)
}

fn scalar(i: &str) -> PResult<TotValue> {
    todo!()
}

fn key_value(i: &str) -> PResult<(String, TotValue)> {
    delimited(
        tuple((multispace0, strip_comments, multispace0)),
        separated_pair(
            key,
            tuple((multispace0, strip_comments, multispace0)),
            scalar,
        ),
        multispace0,
    )(i)
}

fn line_comment(i: &str) -> PResult<()> {
    value((), pair(tag("//"), is_not("\r\n")))(i)
}

fn block_comment(i: &str) -> PResult<()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(i)
}

fn strip_comments(i: &str) -> PResult<()> {
    alt((line_comment, block_comment))(i)
}

fn parse(i: &str) -> Result<TotValue, TotError> {
    // TODO stub

    todo!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn adhoc() {
        // TODO i really hope this isn't the answer
        fn a(i: &str) -> super::PResult<&str> {
            super::delimited(
                super::tuple((
                    super::multispace0,
                    super::strip_comments,
                    super::multispace0,
                )),
                super::take_until(" "),
                super::tuple((
                    super::multispace0,
                    super::strip_comments,
                    super::multispace0,
                )),
            )(i)
        }

        let (rem, par) = a("/*beep*/ hello //world").unwrap();
        assert_eq!(par, "hello");
    }

    #[test]
    fn id() {
        let (rem, par) = super::id("my-key 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");
    }

    #[test]
    fn key() {
        let (rem, par) = super::key("my-key 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");

        let (rem, par) = super::key("\"my-key\" 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");

        let (rem, par) = super::key("\"my key\" 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my key");
    }

    #[test]
    fn line_comment() {
        let (rem, _) = super::line_comment("// blah").unwrap();
        assert_eq!(rem, "");

        let (rem, _) = super::line_comment("// this is a comment\ntext").unwrap();
        assert_eq!(rem, "\ntext");
    }

    #[test]
    fn block_comment() {
        let (rem, _) = super::block_comment("/* moo */\nhello world").unwrap();
        assert_eq!(rem, "\nhello world");

        let (rem, _) = super::block_comment("/* moo\n\n\t\r */\nhello world").unwrap();
        assert_eq!(rem, "\nhello world");
    }
}
