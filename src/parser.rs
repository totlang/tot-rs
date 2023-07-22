use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_till1, take_until},
    character::complete::multispace1,
    combinator::{map, value},
    multi::many0,
    number::complete::double,
    sequence::{delimited, pair, separated_pair, tuple},
    IResult,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error ocurred while parsing")]
    ParseError,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TotValue {
    Unit,
    Boolean(bool),
    String(String),
    Number(f64),
    List(Vec<TotValue>),
    Dict(HashMap<String, TotValue>),
}

pub type PResult<'a, T> = IResult<&'a str, T>;

fn token(i: &str) -> PResult<&str> {
    take_till1(|c: char| c.is_whitespace())(i)
}

pub(crate) fn unit(i: &str) -> PResult<()> {
    value((), tag("null"))(i)
}

pub(crate) fn boolean(i: &str) -> PResult<bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(i)
}

pub(crate) fn number(i: &str) -> PResult<f64> {
    double(i)
}

pub(crate) fn string(i: &str) -> PResult<String> {
    map(
        delimited(tag("\""), take_till(|c: char| c == '"'), tag("\"")),
        String::from,
    )(i)
}

fn whitespace(i: &str) -> PResult<()> {
    map(multispace1, |_| ())(i)
}

fn comma(i: &str) -> PResult<()> {
    value((), tag(","))(i)
}

fn line_comment(i: &str) -> PResult<()> {
    value((), pair(tag("//"), is_not("\r\n")))(i)
}

fn block_comment(i: &str) -> PResult<()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(i)
}

pub(crate) fn all_ignored(i: &str) -> PResult<()> {
    map(
        many0(alt((line_comment, block_comment, comma, whitespace))),
        |_| (),
    )(i)
}

fn list(i: &str) -> PResult<TotValue> {
    delimited(tag("["), list_contents, tag("]"))(i)
}

fn list_contents(i: &str) -> PResult<TotValue> {
    map(many0(delimited(all_ignored, scalar, all_ignored)), |v| {
        TotValue::List(v)
    })(i)
}

fn dict(i: &str) -> PResult<TotValue> {
    delimited(tag("{"), dict_contents, tag("}"))(i)
}

fn dict_contents(i: &str) -> PResult<TotValue> {
    map(many0(key_value), |v| TotValue::Dict(HashMap::from_iter(v)))(i)
}

pub(crate) fn key(i: &str) -> PResult<String> {
    alt((map(string, String::from), map(token, String::from)))(i)
}

pub(crate) fn expression(i: &str) -> PResult<TotValue> {
    todo!()
}

// TODO missing s-expressions
fn scalar(i: &str) -> PResult<TotValue> {
    alt((
        map(unit, |_| TotValue::Unit),
        map(boolean, |v| TotValue::Boolean(v)),
        map(number, |v| TotValue::Number(v)),
        map(string, |v| TotValue::String(v)),
        list,
        dict,
    ))(i)
}

fn key_value(i: &str) -> PResult<(String, TotValue)> {
    delimited(
        all_ignored,
        separated_pair(key, all_ignored, scalar),
        all_ignored,
    )(i)
}

pub fn parse(i: &str) -> Result<TotValue, Error> {
    if let Ok((rem, v)) = dict_contents(i) {
        if rem.is_empty() {
            return Ok(v);
        }
    }

    Err(Error::ParseError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse() {
        if let TotValue::Dict(v) = parse("test 1").unwrap() {
            assert_eq!(
                v.get_key_value("test").unwrap(),
                (&"test".to_string(), &TotValue::Number(1.0))
            );
        } else {
            assert!(false);
        }

        if let TotValue::Dict(v) = parse("test 1 blah true").unwrap() {
            assert_eq!(v.get("test").unwrap(), &TotValue::Number(1.0));
            assert_eq!(v.get("blah").unwrap(), &TotValue::Boolean(true));
        } else {
            assert!(false);
        }

        if let TotValue::Dict(v) = parse(
            "\
test 1
blah true
dict {
    hello \"world\"
}
",
        )
        .unwrap()
        {
            assert_eq!(v.get("test").unwrap(), &TotValue::Number(1.0));
            assert_eq!(v.get("blah").unwrap(), &TotValue::Boolean(true));
            assert_eq!(
                v.get("dict").unwrap(),
                &TotValue::Dict({
                    let mut m = HashMap::new();
                    m.insert("hello".to_string(), TotValue::String("world".to_string()));

                    m
                })
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_unit() {
        let (rem, _) = unit("null// hello").unwrap();
        assert_eq!(rem, "// hello");
    }

    #[test]
    fn test_token() {
        let (rem, par) = token("my-key 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");

        assert!(token("").is_err());
    }

    #[test]
    fn test_boolean() {
        let (_, par) = boolean("true").unwrap();
        assert_eq!(par, true);

        let (_, par) = boolean("false").unwrap();
        assert_eq!(par, false);

        assert!(boolean("True").is_err());
        assert!(boolean("False").is_err());
        assert!(boolean("").is_err());
    }

    #[test]
    fn test_number() {
        let (_, par) = number("10").unwrap();
        assert_eq!(par, f64::from(10));

        let (_, par) = number("10.").unwrap();
        assert_eq!(par, f64::from(10));

        let (_, par) = number("10.0").unwrap();
        assert_eq!(par, f64::from(10));

        let (_, par) = number("0").unwrap();
        assert_eq!(par, f64::from(0));

        let (_, par) = number("0.1").unwrap();
        assert_eq!(par, f64::from(0.1));

        let (_, par) = number(".1").unwrap();
        assert_eq!(par, f64::from(0.1));

        let (_, par) = number("10]").unwrap();
        assert_eq!(par, f64::from(10));

        assert!(number("one").is_err());
        assert!(number("").is_err());
    }

    #[test]
    fn test_string() {
        let (rem, par) = string("\"hello world\"foo").unwrap();
        assert_eq!(rem, "foo");
        assert_eq!(par, "hello world");

        assert!(string("hello world").is_err());
        assert!(string("1").is_err());
    }

    #[test]
    fn test_whitespace() {
        let (rem, _) = whitespace(" hello").unwrap();
        assert_eq!(rem, "hello");

        let (rem, _) = whitespace(" ").unwrap();
        assert_eq!(rem, "");

        assert!(whitespace("hello").is_err());
    }

    #[test]
    fn test_line_comment() {
        let (rem, _) = line_comment("// blah").unwrap();
        assert_eq!(rem, "");

        let (rem, _) = line_comment("// this is a comment\ntext").unwrap();
        assert_eq!(rem, "\ntext");
    }

    #[test]
    fn test_block_comment() {
        let (rem, _) = block_comment("/* moo */\nhello world").unwrap();
        assert_eq!(rem, "\nhello world");

        let (rem, _) = block_comment("/* moo\n\n\t\r */\nhello world").unwrap();
        assert_eq!(rem, "\nhello world");
    }

    #[test]
    fn test_all_ignored() {
        let (rem, _) = all_ignored("/* hello world *///hello").unwrap();
        assert_eq!(rem, "");

        let (rem, _) = all_ignored("/* hello world */    //hello").unwrap();
        assert_eq!(rem, "");

        let (rem, _) = all_ignored("//hello /* hello world */").unwrap();
        assert_eq!(rem, "");

        let (rem, _) = all_ignored("/* hello world */ woot").unwrap();
        assert_eq!(rem, "woot");
    }

    #[test]
    fn test_list() {
        let (rem, par) = list("[]").unwrap();
        assert_eq!(rem, "");
        assert_eq!(par, TotValue::List(vec![]));

        let (rem, par) = list("[1]").unwrap();
        assert_eq!(rem, "");
        assert_eq!(par, TotValue::List(vec![TotValue::Number(1.0)]));

        let (rem, par) = list("[] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(par, TotValue::List(vec![]));

        let (rem, par) = list("[1] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(par, TotValue::List(vec![TotValue::Number(1.0)]));

        let (rem, par) = list("[1, 2\n , /* inner comment */ 3.1 4] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(
            par,
            TotValue::List(vec![
                TotValue::Number(1.0),
                TotValue::Number(2.0),
                TotValue::Number(3.1),
                TotValue::Number(4.0)
            ])
        );

        assert!(list("").is_err());
        // Not a list
        assert!(list("hello").is_err());
        // Invalid identifier
        assert!(list("[hello]").is_err());
        // Unterminated list
        assert!(list("[").is_err());
        assert!(list("[ 1 ").is_err());
    }

    #[test]
    fn test_dict() {
        let (rem, par) = dict("{}").unwrap();
        assert_eq!(rem, "");
        assert_eq!(par, TotValue::Dict(HashMap::default()));

        let (_, par) = dict("{hello \"world\"}").unwrap();
        assert_eq!(
            par,
            TotValue::Dict({
                let mut map = HashMap::new();
                map.insert("hello".to_string(), TotValue::String("world".to_string()));

                map
            })
        );

        let (_, par) = dict("{hello \"world\" inner-list [true 10]}").unwrap();
        assert_eq!(
            par,
            TotValue::Dict({
                let mut map = HashMap::new();
                map.insert("hello".to_string(), TotValue::String("world".to_string()));
                map.insert(
                    "inner-list".to_string(),
                    TotValue::List(vec![TotValue::Boolean(true), TotValue::Number(10.0)]),
                );

                map
            })
        );
    }

    #[test]
    fn test_key() {
        let (rem, par) = key("my-key").unwrap();
        assert_eq!(rem, "");
        assert_eq!(par, "my-key");

        let (rem, par) = key("my-key 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");

        let (rem, par) = key("\"my-key\" 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my-key");

        let (rem, par) = key("\"my key\" 2").unwrap();
        assert_eq!(rem, " 2");
        assert_eq!(par, "my key");
    }

    #[test]
    fn test_scalar() {
        let (_, par) = scalar("true").unwrap();
        assert_eq!(par, TotValue::Boolean(true));

        let (_, par) = scalar("1").unwrap();
        assert_eq!(par, TotValue::Number(1.0));

        let (_, par) = scalar("\"hello\"").unwrap();
        assert_eq!(par, TotValue::String("hello".to_string()));

        let (_, par) = scalar("[false]").unwrap();
        assert_eq!(par, TotValue::List(vec![TotValue::Boolean(false)]));
    }

    #[test]
    fn test_key_value() {
        let (_, par) = key_value("hello true").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(par.1, TotValue::Boolean(true));

        let (_, par) = key_value("\"hello world\" false").unwrap();
        assert_eq!(par.0, "hello world");
        assert_eq!(par.1, TotValue::Boolean(false));

        let (_, par) = key_value("hello 10").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(par.1, TotValue::Number(10.0));

        let (_, par) = key_value("hello \"world\"").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(par.1, TotValue::String("world".to_string()));

        let (_, par) = key_value("hello [0 true [\"hello\"]]").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(
            par.1,
            TotValue::List(vec![
                TotValue::Number(0.0),
                TotValue::Boolean(true),
                TotValue::List(vec![TotValue::String("hello".to_string())])
            ])
        );
    }
}
