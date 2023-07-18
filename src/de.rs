use serde::de::{MapAccess, SeqAccess};
use serde::{de, Deserialize};

use crate::error::{Error, Result};
use crate::parser;

// TODO July 17, 2023 Tim: integers are rounded when deserializing, check that this is okay

#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de str,
    depth: u64,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input, depth: 0 }
    }

    fn peek(&self) -> Result<char> {
        self.input
            .chars()
            .next()
            .ok_or(Error::SerdeError("eof".to_string()))
    }

    fn take(&mut self) -> Result<char> {
        let c = self.peek()?;
        self.input = &self.input[c.len_utf8()..];

        Ok(c)
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

    fn parse_key(&mut self) -> Result<String> {
        let (rem, par) = parser::key(self.input).map_err(|e| Error::SerdeError(e.to_string()))?;

        self.input = rem;

        Ok(par)
    }
}

/// Try to deserialize a `str` into a `T`.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    let _ = deserializer.parse_ws(); // Remove trailing newline just in case
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::SerdeError("Input not empty".to_string()))
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
        visitor.visit_i8(i8::try_from(self.parse_number()?.round() as i64)?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i16(i16::try_from(self.parse_number()?.round() as i64)?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i32(i32::try_from(self.parse_number()?.round() as i64)?)
    }

    // TODO: this less fallible than smaller integers because we do a raw cast to i64
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_number()?.round() as i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u8(u8::try_from(self.parse_number()?.round() as u64)?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u16(u16::try_from(self.parse_number()?.round() as u64)?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(u32::try_from(self.parse_number()?.round() as u64)?)
    }

    // TODO: this less fallible than smaller integers because we do a raw cast to u64
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_number()?.round() as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_number()? as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    // TODO needs tests
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
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

    // TODO needs tests
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    // TODO needs tests
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    // TODO needs tests
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    // TODO needs tests
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.take()? == '[' {
            self.depth += 1;
            let val = visitor.visit_seq(Access::new(self))?;
            self.depth -= 1;
            if self.take()? == ']' {
                Ok(val)
            } else {
                Err(Error::SerdeError("Expected array end".to_string()))
            }
        } else {
            Err(Error::SerdeError("Expected array open".to_string()))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.depth < 1 {
            self.parse_ws()?;

            self.depth += 1;
            let val = visitor.visit_map(Access::new(self))?;
            self.depth -= 1;

            Ok(val)
        } else {
            if self.take()? == '{' {
                self.depth += 1;
                let val = visitor.visit_map(Access::new(self))?;
                self.depth -= 1;
                if self.take()? == '}' {
                    Ok(val)
                } else {
                    Err(Error::SerdeError("Expected dict end".to_string()))
                }
            } else {
                Err(Error::SerdeError("Expected dict open".to_string()))
            }
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    // TODO needs tests
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
        visitor.visit_string(self.parse_key()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct KeyDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> KeyDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut KeyDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.de.parse_key()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_bytes(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_byte_buf(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_option(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::SerdeError("a tuple cannot be a key".to_string()))
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
        self.de.deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_map(visitor)
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
        self.de.deserialize_struct(name, fields, visitor)
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
        self.de.deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_ignored_any(visitor)
    }
}

struct Access<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Access<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Access { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for Access<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.de.parse_ws()?;
        if self.de.peek()? == ']' {
            return Ok(None);
        }
        let r = seed.deserialize(&mut *self.de).map(Some);
        if r.is_ok() {
            self.de.parse_ws()?;
        }

        r
    }
}

impl<'de, 'a> MapAccess<'de> for Access<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.de.parse_ws()?;
        if self.de.depth > 1 && self.de.peek()? == '}' {
            return Ok(None);
        }
        let r = seed
            .deserialize(&mut KeyDeserializer::new(&mut self.de))
            .map(Some);
        if r.is_ok() {
            self.de.parse_ws()?;
        } else if self.de.depth < 2 {
            // We ran out of keys to parse
            let _ = self.de.parse_ws();
            return Ok(None);
        }

        r
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.parse_ws()?;
        let r = seed.deserialize(&mut *self.de);
        if r.is_ok() {
            self.de.parse_ws()?;
        }

        r
    }
}

#[cfg(test)]
mod tests {
    use super::{from_str, Deserializer};

    mod deserializer_tests {
        use super::*;

        fn de(i: &str) -> Deserializer {
            Deserializer::from_str(i)
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

    mod de_tests {
        use super::*;
        use serde::Deserialize;

        #[test]
        fn test_de_bool() {
            assert_eq!(from_str::<bool>("true").unwrap(), true);
            assert_eq!(from_str::<bool>("false").unwrap(), false);

            assert!(from_str::<bool>("1.0").is_err());
        }

        mod ints {
            use super::*;

            #[test]
            fn test_de_i8() {
                assert_eq!(from_str::<i8>("-1").unwrap(), -1);
                assert_eq!(from_str::<i8>("0").unwrap(), 0);
                assert_eq!(from_str::<i8>("1").unwrap(), 1);

                assert_eq!(from_str::<i8>("127").unwrap(), 127);
                assert_eq!(from_str::<i8>("-128").unwrap(), -128);

                assert!(from_str::<i8>("true").is_err());
                assert!(from_str::<i8>("128").is_err());
                assert!(from_str::<i8>("-129").is_err());
            }

            #[test]
            fn test_de_i16() {
                assert_eq!(from_str::<i16>("-1").unwrap(), -1);
                assert_eq!(from_str::<i16>("0").unwrap(), 0);
                assert_eq!(from_str::<i16>("1").unwrap(), 1);

                assert_eq!(from_str::<i16>("32767").unwrap(), 32767);
                assert_eq!(from_str::<i16>("-32768").unwrap(), -32768);

                assert!(from_str::<i16>("true").is_err());
                assert!(from_str::<i16>("32768").is_err());
                assert!(from_str::<i16>("-32769").is_err());
            }

            #[test]
            fn test_de_i32() {
                assert_eq!(from_str::<i32>("-1").unwrap(), -1);
                assert_eq!(from_str::<i32>("0").unwrap(), 0);
                assert_eq!(from_str::<i32>("1").unwrap(), 1);

                assert_eq!(from_str::<i32>("2147483647").unwrap(), 2147483647);
                assert_eq!(from_str::<i32>("-2147483648").unwrap(), -2147483648);

                assert!(from_str::<i32>("true").is_err());
                assert!(from_str::<i32>("2147483648").is_err());
                assert!(from_str::<i32>("-2147483649").is_err())
            }

            #[test]
            fn test_de_i64() {
                assert_eq!(from_str::<i64>("-1").unwrap(), -1);
                assert_eq!(from_str::<i64>("0").unwrap(), 0);
                assert_eq!(from_str::<i64>("1").unwrap(), 1);

                assert_eq!(
                    from_str::<i64>("9223372036854775807").unwrap(),
                    9223372036854775807
                );
                assert_eq!(
                    from_str::<i64>("-9223372036854775808").unwrap(),
                    -9223372036854775808
                );

                assert!(from_str::<i64>("true").is_err());
            }

            #[test]
            fn test_de_i64_truncate() {
                assert_eq!(
                    from_str::<i64>("9223372036854775809").unwrap(),
                    9223372036854775807
                );
            }
        }

        mod unsigned_ints {
            use super::*;

            #[test]
            fn test_de_u8() {
                assert_eq!(from_str::<u8>("0").unwrap(), 0);
                assert_eq!(from_str::<u8>("1").unwrap(), 1);

                assert_eq!(from_str::<u8>("255").unwrap(), 255);

                assert!(from_str::<u8>("true").is_err());
                assert!(from_str::<u8>("256").is_err());
            }

            #[test]
            fn test_de_u16() {
                assert_eq!(from_str::<u16>("0").unwrap(), 0);
                assert_eq!(from_str::<u16>("1").unwrap(), 1);

                assert_eq!(from_str::<u16>("65535").unwrap(), 65535);

                assert!(from_str::<u16>("true").is_err());
                assert!(from_str::<u16>("65536").is_err());
            }

            #[test]
            fn test_de_u32() {
                assert_eq!(from_str::<u32>("0").unwrap(), 0);
                assert_eq!(from_str::<u32>("1").unwrap(), 1);

                assert_eq!(from_str::<u32>("4294967295").unwrap(), 4294967295);

                assert!(from_str::<u32>("true").is_err());
                assert!(from_str::<u32>("4294967296").is_err());
            }

            #[test]
            fn test_de_u64() {
                assert_eq!(from_str::<u64>("0").unwrap(), 0);
                assert_eq!(from_str::<u64>("1").unwrap(), 1);

                assert_eq!(
                    from_str::<u64>("18446744073709551615").unwrap(),
                    18446744073709551615
                );

                assert!(from_str::<u64>("true").is_err());
            }

            #[test]
            fn test_de_u64_truncate() {
                assert_eq!(
                    from_str::<u64>("18446744073709551616").unwrap(),
                    18446744073709551615
                );
            }

            #[test]
            fn test_de_unsigned_truncate() {
                assert_eq!(from_str::<u8>("-3").unwrap(), 0);
                assert_eq!(from_str::<u16>("-3").unwrap(), 0);
                assert_eq!(from_str::<u32>("-3").unwrap(), 0);
                assert_eq!(from_str::<u64>("-3").unwrap(), 0);
            }
        }

        mod floats {
            use super::*;

            #[test]
            fn test_de_f32() {
                assert_eq!(from_str::<f32>("-1").unwrap(), -1.0);
                assert_eq!(from_str::<f32>("0").unwrap(), 0.0);
                assert_eq!(from_str::<f32>("1").unwrap(), 1.0);

                assert_eq!(from_str::<f32>("2147483647").unwrap(), 2147483647.0);
                assert_eq!(from_str::<f32>("-2147483648").unwrap(), -2147483648.0);

                assert!(from_str::<f32>("true").is_err());
            }

            #[test]
            fn test_de_f64() {
                assert_eq!(from_str::<f64>("-1").unwrap(), -1.0);
                assert_eq!(from_str::<f64>("0").unwrap(), 0.0);
                assert_eq!(from_str::<f64>("1").unwrap(), 1.0);

                assert_eq!(
                    from_str::<f64>("9223372036854775807").unwrap(),
                    9223372036854775807.0
                );
                assert_eq!(
                    from_str::<f64>("-9223372036854775808").unwrap(),
                    -9223372036854775808.0
                );

                assert!(from_str::<f64>("true").is_err());
            }

            #[test]
            fn test_de_float_truncate() {
                assert!(from_str::<f32>("123819023801928309128301231234218309812408210").is_ok());
                assert!(from_str::<f64>("123819023801928309128301231234218309812408210").is_ok());
            }
        }

        #[test]
        fn test_de_string() {
            assert_eq!(
                from_str::<String>("\"hello world\"").unwrap(),
                "hello world"
            );
        }

        #[test]
        fn test_de_list() {
            assert_eq!(from_str::<Vec<bool>>("[\ntrue\n]").unwrap(), vec![true]);
            assert_eq!(
                from_str::<Vec<bool>>(
                    "\
[
    true
    false
    true
]
"
                )
                .unwrap(),
                vec![true, false, true]
            );
            assert_eq!(
                from_str::<Vec<i8>>(
                    "\
[
    -2
    64
    0
]
"
                )
                .unwrap(),
                vec![-2, 64, 0]
            );
        }

        #[test]
        fn test_de_tuple() {
            assert_eq!(
                from_str::<(bool, i8, String)>(
                    "\
[
    true
    22
    \"hello\"
]
"
                )
                .unwrap(),
                (true, 22, "hello".to_string())
            );
        }

        #[test]
        fn test_de_map() {
            use std::collections::HashMap;

            let dict = from_str::<HashMap<String, i8>>(
                "\
\"hello\" 101
\"world\" -2
",
            )
            .unwrap();

            assert_eq!(dict, {
                let mut m = HashMap::new();
                m.insert("hello".to_string(), 101);
                m.insert("world".to_string(), -2);

                m
            });
            assert_eq!(dict.get(&"hello".to_string()).unwrap(), &101);
            assert_eq!(dict.get(&"world".to_string()).unwrap(), &-2);
        }

        mod structs {
            use super::*;
            use std::collections::HashMap;

            #[test]
            fn test_de_newtype_struct_int() {
                #[derive(Deserialize)]
                struct TestStruct(i32);

                assert_eq!(from_str::<TestStruct>("10").unwrap().0, 10);
            }

            #[test]
            fn test_de_newtype_struct_map() {
                #[derive(Deserialize)]
                struct TestStruct(HashMap<String, String>);

                assert_eq!(
                    from_str::<TestStruct>("hello \"world\"")
                        .unwrap()
                        .0
                        .get(&"hello".to_string())
                        .unwrap(),
                    &"world".to_string()
                );
                assert_eq!(
                    from_str::<TestStruct>("\"hello\" \"world\"")
                        .unwrap()
                        .0
                        .get(&"hello".to_string())
                        .unwrap(),
                    &"world".to_string()
                );
            }

            #[test]
            fn test_de_tuple_struct_int_int() {
                #[derive(Deserialize)]
                struct TestStruct(i32, i32);

                let r = from_str::<TestStruct>("[10 -123]").unwrap();

                assert_eq!(r.0, 10);
                assert_eq!(r.1, -123);
            }

            #[test]
            fn test_de_tuple_struct_string_bool() {
                #[derive(Deserialize)]
                struct TestStruct(String, bool);

                let r = from_str::<TestStruct>("[\"hello world\" false]").unwrap();

                assert_eq!(r.0, "hello world".to_string());
                assert_eq!(r.1, false);
            }

            #[test]
            fn test_de_nested_tuple_struct() {
                #[derive(Deserialize)]
                struct TestStruct(Inner, Inner);

                #[derive(Deserialize)]
                struct Inner(i32, bool);

                let r = from_str::<TestStruct>(
                    "\
[
    [
        -22 true
    ]
    [
        123 false
    ]
]
",
                )
                .unwrap();

                assert_eq!(r.0 .0, -22);
                assert_eq!(r.0 .1, true);
                assert_eq!(r.1 .0, 123);
                assert_eq!(r.1 .1, false);
            }

            #[test]
            fn test_de_simple_struct() {
                #[derive(Deserialize)]
                struct TestStruct {
                    name: String,
                    age: u32,
                    foods: Vec<String>,
                    hungry: bool,
                }

                let r = from_str::<TestStruct>(
                    "\
name \"Tim\"
age 18
foods [
    \"rice\"
    \"chicken\"
]
hungry true
",
                )
                .unwrap();
                assert_eq!(r.name, "Tim".to_string());
                assert_eq!(r.age, 18);
                assert_eq!(r.foods[0], "rice".to_string());
                assert_eq!(r.foods[1], "chicken".to_string());
                assert_eq!(r.hungry, true);
            }

            #[test]
            fn test_de_nested_struct() {
                #[derive(Deserialize)]
                struct TestStruct {
                    name: String,
                    inner: Inner,
                }

                #[derive(Deserialize)]
                struct Inner {
                    age: u32,
                    bools: Vec<bool>,
                }

                let r = from_str::<TestStruct>(
                    "\
name \"Tim\"
inner {
    age 100
    bools [
        true
        false
    ]
}
",
                )
                .unwrap();

                assert_eq!(r.name, "Tim".to_string());
                assert_eq!(r.inner.age, 100);
                assert_eq!(r.inner.bools[0], true);
                assert_eq!(r.inner.bools[1], false);
            }
        }
    }
}
