use std::cell::Cell;

use serde::{de, Deserialize};

use crate::error::{Error, Result};
use crate::parser;

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de str,
    offset: Cell<usize>,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        let offset = Cell::new(0);

        Deserializer { input, offset }
    }

    fn peek(&self) -> Result<char> {
        self.input
            .chars()
            .next()
            .ok_or(Error::SerdeError("eof".to_string()))
    }

    fn parse_ws(&mut self) -> Result<()> {
        let (rem, _) =
            parser::all_ignored(self.input).map_err(|e| Error::SerdeError(e.to_string()))?;

        self.input = rem;

        Ok(())
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let (rem, par) =
            parser::boolean(self.input).map_err(|e| Error::SerdeError(e.to_string()))?;

        self.input = rem;

        Ok(par)
    }

    fn parse_number(&mut self) -> Result<f64> {
        let (rem, par) =
            parser::number(self.input).map_err(|e| Error::SerdeError(e.to_string()))?;

        self.input = rem;

        Ok(par)
    }

    fn parse_string(&mut self) -> Result<String> {
        let (rem, par) =
            parser::string(self.input).map_err(|e| Error::SerdeError(e.to_string()))?;

        self.input = rem;

        Ok(par)
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::SerdeError("".to_string()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.peek()? {
            't' | 'f' => self.deserialize_bool(visitor),
            '0'..='9' | '-' => self.deserialize_f64(visitor),
            '"' | '\'' => self.deserialize_str(visitor),
            _ => Err(Error::SerdeError("Syntax".to_string())),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.parse_string()?.as_str())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{from_str, Deserializer};

    fn de(i: &str) -> Deserializer {
        Deserializer::from_str(i)
    }

    #[test]
    fn test_de_bool() {
        assert_eq!(from_str::<bool>("true").unwrap(), true);
    }

    #[test]
    fn test_de_string() {
        assert_eq!(
            from_str::<String>("\"hello world\"").unwrap(),
            "hello world"
        );
    }

    #[test]
    fn test_whitespace() {
        let input = "/*comment */ true //asdf";

        let mut de = de(input);

        assert!(de.parse_bool().is_err());

        de.parse_ws().unwrap();

        assert_eq!(de.parse_bool().unwrap(), true);
    }

    #[test]
    fn test_boolean() {
        let mut de = de("true");
        assert_eq!(de.parse_bool().unwrap(), true);
    }

    #[test]
    fn test_number_float() {
        let mut de = de("1.0");
        assert_eq!(de.parse_number().unwrap(), 1.0);
    }

    #[test]
    fn test_string() {
        let mut de = de("\"hello world\"");
        assert_eq!(de.parse_string().unwrap(), "hello world");
    }
}
