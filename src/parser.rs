use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_till1, take_until},
    character::complete::{char, digit1, multispace1},
    combinator::{map, map_res, not, opt, value},
    multi::{many0, many1},
    number::complete::double,
    sequence::{delimited, pair, separated_pair, terminated, tuple},
    IResult,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error ocurred while parsing")]
    ParseError,
    #[error("whitespace error {0}")]
    WhitespaceError(String),
    #[error("unit error {0}")]
    UnitError(String),
    #[error("bool error {0}")]
    BoolError(String),
    #[error("integer error {0}")]
    IntegerError(String),
    #[error("float error {0}")]
    FloatError(String),
    #[error("string error {0}")]
    StringError(String),
    #[error("expression error {0}")]
    ExpressionError(String),
    #[error("dict error {0}")]
    DictError(String),
    #[error("list error {0}")]
    ListError(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TotValue {
    Unit,
    Boolean(bool),
    String(String),
    Integer(i64),
    Float(f64),
    List(Vec<TotValue>),
    Dict(HashMap<String, TotValue>),
    Generator { name: String },
    Missing, // TODO probably should add more context data?
}

#[derive(Debug)]
pub(crate) struct Parser<'a> {
    known_expressions: RefCell<HashMap<&'a str, TotExpression<'a>>>,
    missing_expressions: RefCell<HashMap<&'a str, TotExpression<'a>>>,
}

// TODO parse_* might be able to be &mut to simplify input access
impl<'a> Parser<'a> {
    pub(crate) fn new() -> Self {
        Self {
            known_expressions: RefCell::new(HashMap::new()),
            missing_expressions: RefCell::new(HashMap::new()),
        }
    }

    // fn data(&self) -> &str {
    //     &self.input[self.offset.get()..]
    // }

    // fn offset(&self, input: &'a str) {
    //     let offset = input.as_ptr() as usize - self.input.as_ptr() as usize;
    //     self.offset.set(offset);
    // }

    fn token(&self, i: &'a str) -> PResult<&str> {
        take_till1(|c: char| c.is_whitespace())(i)
    }

    pub(crate) fn unit(&self, i: &'a str) -> PResult<()> {
        value((), tag("null"))(i)
    }

    pub(crate) fn boolean(&self, i: &'a str) -> PResult<bool> {
        alt((value(true, tag("true")), value(false, tag("false"))))(i)
    }

    pub(crate) fn integer(&'a self, i: &'a str) -> PResult<i64> {
        map_res(
            terminated(
                tuple((opt(char('-')), digit1)),
                not(|i: &'a str| self.float(i)),
            ),
            |(sign, v): (Option<char>, &str)| match sign {
                Some(sign) if sign == '-' => v
                    .parse()
                    .map(|parsed_int: i64| parsed_int * -1)
                    .map_err(|_| Error::IntegerError(format!("Cannot parse {sign}{v}"))),
                Some(sign) => Err(Error::IntegerError(format!("Unhandled sign {sign}"))),
                None => v
                    .parse()
                    .map_err(|_| Error::IntegerError(format!("Cannot parse {v}"))),
            },
        )(i)
    }

    pub(crate) fn float(&self, i: &'a str) -> PResult<f64> {
        double(i)
    }

    pub(crate) fn string(&self, i: &'a str) -> PResult<String> {
        map(
            delimited(tag("\""), take_till(|c: char| c == '"'), tag("\"")),
            String::from,
        )(i)
    }

    fn whitespace(&self, i: &'a str) -> PResult<()> {
        map(multispace1, |_| ())(i)
    }

    fn comma(&self, i: &'a str) -> PResult<()> {
        value((), tag(","))(i)
    }

    fn line_comment(&self, i: &'a str) -> PResult<()> {
        value((), pair(tag("//"), is_not("\r\n")))(i)
    }

    fn block_comment(&self, i: &'a str) -> PResult<()> {
        value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(i)
    }

    pub(crate) fn all_ignored(&'a self, i: &'a str) -> PResult<()> {
        map(
            many0(alt((
                |i: &'a str| self.line_comment(i),
                |i: &'a str| self.block_comment(i),
                |i: &'a str| self.comma(i),
                |i: &'a str| self.whitespace(i),
            ))),
            |_| (),
        )(i)
    }

    fn list(&'a self, i: &'a str) -> PResult<TotValue> {
        delimited(
            tag("["),
            map(
                many0(delimited(
                    |i: &'a str| self.all_ignored(i),
                    |i: &'a str| self.scalar(i),
                    |i: &'a str| self.all_ignored(i),
                )),
                |v| TotValue::List(v),
            ),
            tag("]"),
        )(i)
    }

    fn dict(&'a self, i: &'a str) -> PResult<TotValue> {
        delimited(tag("{"), |i: &'a str| self.dict_contents(i), tag("}"))(i)
    }

    fn dict_contents(&'a self, i: &'a str) -> PResult<TotValue> {
        map(many0(|i: &'a str| self.key_value(i)), |v| {
            TotValue::Dict(HashMap::from_iter(v))
        })(i)
    }

    pub(crate) fn key(&'a self, i: &'a str) -> PResult<String> {
        alt((
            map(|i: &'a str| self.string(i), String::from),
            map(|i: &'a str| self.token(i), String::from),
        ))(i)
    }

    pub(crate) fn expression(&'a self, i: &'a str) -> PResult<TotValue> {
        delimited(
            tag("("),
            delimited(
                |i: &'a str| self.all_ignored(i),
                alt((
                    |i: &'a str| self.math_exp(i),
                    |i: &'a str| self.ref_exp(i),
                    |i: &'a str| self.gen_def_exp(i),
                    |i: &'a str| self.gen_use_exp(i),
                )),
                |i: &'a str| self.all_ignored(i),
            ),
            tag(")"),
        )(i)
    }

    fn math_exp(&self, i: &'a str) -> PResult<TotValue> {
        todo!()
    }

    fn ref_exp(&self, i: &'a str) -> PResult<TotValue> {
        todo!()
    }

    fn gen_def_exp(&self, i: &'a str) -> PResult<TotValue> {
        todo!()
    }

    fn gen_use_exp(&self, i: &'a str) -> PResult<TotValue> {
        todo!()
    }

    fn scalar(&'a self, i: &'a str) -> PResult<TotValue> {
        alt((
            map(|i: &'a str| self.unit(i), |_| TotValue::Unit),
            map(|i: &'a str| self.boolean(i), |v| TotValue::Boolean(v)),
            map(|i: &'a str| self.integer(i), |v| TotValue::Integer(v)),
            map(|i: &'a str| self.float(i), |v| TotValue::Float(v)),
            map(|i: &'a str| self.string(i), |v| TotValue::String(v)),
            |i: &'a str| self.list(i),
            |i: &'a str| self.dict(i),
        ))(i)
    }

    fn key_value(&'a self, i: &'a str) -> PResult<(String, TotValue)> {
        delimited(
            |i: &'a str| self.all_ignored(i),
            separated_pair(
                |i: &'a str| self.key(i),
                |i: &'a str| self.all_ignored(i),
                |i: &'a str| self.scalar(i),
            ),
            |i: &'a str| self.all_ignored(i),
        )(i)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TotExpression<'a> {
    Unit,
    Ref {
        name: &'a str,
        accessors: Vec<&'a str>,
    },
    Add,
    Sub,
    Mul,
    Div,
    For,
}

type Result<T> = std::result::Result<T, Error>;
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

pub(crate) fn integer<'a>(i: &'a str) -> PResult<i64> {
    map_res(
        terminated(tuple((opt(char('-')), digit1)), not(|i: &'a str| float(i))),
        |(sign, v): (Option<char>, &str)| match sign {
            Some(sign) if sign == '-' => v
                .parse()
                .map(|parsed_int: i64| parsed_int * -1)
                .map_err(|_| Error::IntegerError(format!("Cannot parse {sign}{v}"))),
            Some(sign) => Err(Error::IntegerError(format!("Unhandled sign {sign}"))),
            None => v
                .parse()
                .map_err(|_| Error::IntegerError(format!("Cannot parse {v}"))),
        },
    )(i)
}

pub(crate) fn float(i: &str) -> PResult<f64> {
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

pub(crate) fn all_ignored1(i: &str) -> PResult<()> {
    map(
        many1(alt((line_comment, block_comment, comma, whitespace))),
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

pub(crate) fn expression(i: &str) -> PResult<TotExpression> {
    todo!()
}

// TODO missing s-expressions
fn scalar(i: &str) -> PResult<TotValue> {
    alt((
        map(unit, |_| TotValue::Unit),
        map(boolean, |v| TotValue::Boolean(v)),
        map(integer, |v| TotValue::Integer(v)),
        map(float, |v| TotValue::Float(v)),
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

pub fn parse(i: &str) -> Result<TotValue> {
    let parser = Parser::new();
    if let Ok((rem, v)) = parser.dict_contents(i) {
        if rem.is_empty() {
            return Ok(v);
        }
    }

    if let Ok((rem, v)) = parser.list(i) {
        if rem.is_empty() {
            return Ok(v);
        }
    }

    // if let Ok((rem, v)) = dict_contents(i) {
    //     if rem.is_empty() {
    //         return Ok(v);
    //     }
    // }

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
                (&"test".to_string(), &TotValue::Integer(1))
            );
        } else {
            assert!(false);
        }

        if let TotValue::Dict(v) = parse("test 1 blah true").unwrap() {
            assert_eq!(v.get("test").unwrap(), &TotValue::Integer(1));
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
            assert_eq!(v.get("test").unwrap(), &TotValue::Integer(1));
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
    fn test_integer() {
        let (_, par) = integer("10").unwrap();
        assert_eq!(par, i64::from(10));

        let (_, par) = integer("0").unwrap();
        assert_eq!(par, i64::from(0));

        let (_, par) = integer("10]").unwrap();
        assert_eq!(par, i64::from(10));

        assert!(integer("one").is_err());
        assert!(integer("").is_err());
    }

    #[test]
    fn test_float() {
        let (_, par) = float("10.").unwrap();
        assert_eq!(par, f64::from(10));

        let (_, par) = float("10.0").unwrap();
        assert_eq!(par, f64::from(10));

        let (_, par) = float("0.1").unwrap();
        assert_eq!(par, f64::from(0.1));

        let (_, par) = float(".1").unwrap();
        assert_eq!(par, f64::from(0.1));

        assert!(float("one").is_err());
        assert!(float("").is_err());
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
        assert_eq!(par, TotValue::List(vec![TotValue::Integer(1)]));

        let (rem, par) = list("[] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(par, TotValue::List(vec![]));

        let (rem, par) = list("[1] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(par, TotValue::List(vec![TotValue::Integer(1)]));

        let (rem, par) = list("[1, 2\n , /* inner comment */ 3.1 4] blah []").unwrap();
        assert_eq!(rem, " blah []");
        assert_eq!(
            par,
            TotValue::List(vec![
                TotValue::Integer(1),
                TotValue::Integer(2),
                TotValue::Float(3.1),
                TotValue::Integer(4)
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
                    TotValue::List(vec![TotValue::Boolean(true), TotValue::Integer(10)]),
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
        assert_eq!(par, TotValue::Integer(1));

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
        assert_eq!(par.1, TotValue::Integer(10));

        let (_, par) = key_value("hello \"world\"").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(par.1, TotValue::String("world".to_string()));

        let (_, par) = key_value("hello [0 true [\"hello\"]]").unwrap();
        assert_eq!(par.0, "hello");
        assert_eq!(
            par.1,
            TotValue::List(vec![
                TotValue::Integer(0),
                TotValue::Boolean(true),
                TotValue::List(vec![TotValue::String("hello".to_string())])
            ])
        );
    }
}
