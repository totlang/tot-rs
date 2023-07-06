mod string;

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{
        escaped, is_not, tag, tag_no_case, take_till1, take_until, take_while, take_while_m_n,
    },
    character::complete::{
        alphanumeric0, alphanumeric1, char, digit1, i32, i64, line_ending, multispace0,
        multispace1, none_of, one_of, space0, space1,
    },
    combinator::{cut, map, map_opt, map_res, not, opt, recognize, success, value, verify},
    error::{context, ParseError},
    multi::separated_list0,
    number::complete::double as nom_double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Compare, IResult, InputLength, InputTake, InputTakeAtPosition,
};

use string::{literal, parse_string as string};

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

fn line_comment(i: &str) -> PResult<()> {
    value((), pair(tag("//"), is_not("\r\n")))(i)
}

fn block_comment(i: &str) -> PResult<()> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(i)
}

fn void(i: &str) -> PResult<()> {
    success(())(i)
}

fn boolean(i: &str) -> PResult<bool> {
    alt((value(true, tag("true")), value(false, tag("false"))))(i)
}

fn double(i: &str) -> PResult<f64> {
    nom_double(i)
}

fn list(i: &str) -> PResult<Vec<TotValue>> {
    preceded(
        char('['),
        cut(terminated(
            separated_list0(preceded(multispace0, opt(char(','))), tot_value),
            preceded(multispace0, char(']')),
        )),
    )(i)
}

fn unit_key(i: &str) -> PResult<(&str, TotValue)> {
    let (rem, i) = recognize(not(space0))(i)?;
    println!("recognize rem {rem} par {i}");

    preceded(
        multispace0,
        map(
            take_while(|c: char| {
                println!("{c}");
                !c.is_whitespace()
            }),
            |v| {
                println!("{v}");
                (v, TotValue::Void)
            },
        ),
    )(i)
}

fn key_value(i: &str) -> PResult<(&str, TotValue)> {
    println!("key_value {i}");
    separated_pair(
        preceded(multispace0, alphanumeric1),
        cut(multispace1),
        tot_value,
    )(i)
}

fn dict(i: &str) -> PResult<HashMap<String, TotValue>> {
    preceded(
        char('{'),
        cut(terminated(
            map(
                separated_list0(
                    preceded(multispace0, opt(char(','))),
                    alt((key_value, unit_key)),
                ),
                |tuple_vec| {
                    tuple_vec
                        .into_iter()
                        .map(|(k, v)| (String::from(k), v))
                        .collect()
                },
            ),
            preceded(multispace0, char('}')),
        )),
    )(i)
}

fn tot_value(i: &str) -> PResult<TotValue> {
    preceded(
        multispace0,
        alt((
            map(dict, TotValue::Dict),
            map(list, TotValue::List),
            map(string, TotValue::String),
            map(double, TotValue::Double),
            map(boolean, TotValue::Boolean),
            value(TotValue::Void, void),
        )),
    )(i)
}

fn root(i: &str) -> PResult<TotValue> {
    delimited(
        multispace0,
        alt((map(dict, TotValue::Dict), map(list, TotValue::List))),
        multispace0,
    )(i)
}

#[cfg(test)]
mod tests {
    #[test]
    fn adhoc() {
        //
    }

    #[test]
    fn line_comment() {
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

    #[test]
    fn boolean() {
        let (_, par) = super::boolean("true").unwrap();
        assert_eq!(par, true);

        let (_, par) = super::boolean("false").unwrap();
        assert_eq!(par, false);

        let r = super::boolean("hello");
        assert!(r.is_err());
    }

    // TODO this test might be flaky due to rounding errors?
    #[test]
    fn double() {
        let (_, par) = super::double("2.2").unwrap();
        assert_eq!(par, 2.2);

        let (_, par) = super::double("2").unwrap();
        assert_eq!(par, 2.0);

        assert!(super::double("hello").is_err());
        assert!(super::double("").is_err());
    }

    // TODO broken
    #[test]
    fn list() {
        let (_, par) = super::list("[ 1 2 3]").unwrap();
        assert!(!par.is_empty());
    }

    // TODO broken
    #[test]
    fn unit_key() {
        let (_, (k, v)) = super::unit_key("my-key").unwrap();
        assert_eq!(k, "my-key");
        assert_eq!(v, super::TotValue::Void);

        // let (_, (k, v)) = super::unit_key("my-key 2").unwrap();
        // assert_eq!(k, "my-key");

        assert!(super::unit_key("my-key 2").is_err());
    }

    #[test]
    fn key_value() {
        let (_, (k, v)) = super::key_value("my-key \"my-value\"").unwrap();
        assert_eq!(k, "my-key");
        if let super::TotValue::String(v) = v {
            assert_eq!(v, "my-value");
        } else {
            assert!(false);
        }
    }
}
