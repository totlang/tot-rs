use serde::{
    ser::{self, Impossible},
    Serialize,
};

use crate::error::{Error, Result};

/// Indents are 4 spaces.
const INDENT: &str = "    ";

trait Formatter {
    fn indent(&mut self);
    fn unindent(&mut self);
    fn get_indent(&self) -> usize;

    fn is_root_type_set(&self) -> bool;
    fn set_root_type(&mut self, root_type: RootType);

    fn write_space<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b" ").map_err(Error::Io)
    }

    fn write_newline<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"\n").map_err(Error::Io)
    }

    fn write_indent<W: ?Sized + std::io::Write>(
        &mut self,
        writer: &mut W,
        precalculated_amount: Option<usize>,
    ) -> Result<()> {
        for _ in 1..precalculated_amount.unwrap_or(self.get_indent()) {
            writer.write_all(INDENT.as_bytes()).map_err(Error::Io)?;
        }

        Ok(())
    }

    #[inline]
    fn write_null<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"null").map_err(Error::Io)
    }

    #[inline]
    fn write_bool<W: ?Sized + std::io::Write>(
        &mut self,
        writer: &mut W,
        value: bool,
    ) -> Result<()> {
        writer
            .write_all(if value { b"true" } else { b"false" })
            .map_err(Error::Io)
    }

    #[inline]
    fn write_number<W: ?Sized + std::io::Write>(
        &mut self,
        writer: &mut W,
        value: f64,
    ) -> Result<()> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        writer.write_all(s.as_bytes()).map_err(Error::Io)
    }

    #[inline]
    fn begin_string<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"\"").map_err(Error::Io)
    }

    #[inline]
    fn end_string<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"\"").map_err(Error::Io)
    }

    #[inline]
    fn write_string_fragment<W: ?Sized + std::io::Write>(
        &mut self,
        writer: &mut W,
        value: &str,
    ) -> Result<()> {
        writer.write_all(value.as_bytes()).map_err(Error::Io)
    }

    #[inline]
    fn write_key<W: ?Sized + std::io::Write>(&mut self, writer: &mut W, value: &str) -> Result<()> {
        self.write_indent(writer, None)?;
        writer.write_all(value.as_bytes()).map_err(Error::Io)?;
        self.write_space(writer)
    }

    #[inline]
    fn begin_list<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        if !self.is_root_type_set() {
            self.set_root_type(RootType::List);
            self.indent();
        }

        if self.get_indent() > 0 {
            writer.write_all(b"[\n").map_err(Error::Io)?;
        }
        self.indent();

        Ok(())
    }

    #[inline]
    fn end_list<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        self.unindent();

        let indent = self.get_indent();
        if indent > 0 {
            self.write_indent(writer, Some(indent))?;
            writer.write_all(b"]").map_err(Error::Io)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn begin_dict<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        if !self.is_root_type_set() {
            self.set_root_type(RootType::Dict);
        }

        if self.get_indent() > 0 {
            writer.write_all(b"{\n").map_err(Error::Io)?;
        }
        self.indent();

        Ok(())
    }

    #[inline]
    fn end_dict<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        self.unindent();

        let indent = self.get_indent();
        if indent > 0 {
            self.write_indent(writer, Some(indent))?;

            writer.write_all(b"}").map_err(Error::Io)?;

            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RootType {
    #[default]
    None,
    Dict,
    List,
}

#[derive(Debug, Default)]
pub struct DefaultFormatter {
    indents: usize,
    root_type: RootType,
}

impl Formatter for DefaultFormatter {
    fn indent(&mut self) {
        self.indents += 1;
    }

    fn unindent(&mut self) {
        self.indents -= 1;
    }

    fn get_indent(&self) -> usize {
        self.indents
    }

    fn is_root_type_set(&self) -> bool {
        self.root_type != RootType::None
    }

    fn set_root_type(&mut self, root_type: RootType) {
        self.root_type = root_type;
    }
}

pub struct CompactFormatter {
    root_type: RootType,
}

// TODO reimplement to not insert newlines
impl Formatter for CompactFormatter {
    fn indent(&mut self) {
        // Intentionally blank
    }

    fn unindent(&mut self) {
        // Intentionally blank
    }

    fn get_indent(&self) -> usize {
        0
    }

    fn is_root_type_set(&self) -> bool {
        self.root_type != RootType::None
    }

    fn set_root_type(&mut self, root_type: RootType) {
        self.root_type = root_type;
    }
}

pub struct KeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W: 'a, F: 'a> KeySerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> Self {
        Self { ser }
    }
}

// TODO unsupported ops need better errors
impl<'a, W: std::io::Write, F: Formatter> ser::Serializer for KeySerializer<'a, W, F> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;

    type SerializeMap = Impossible<(), Error>;

    type SerializeStruct = Impossible<(), Error>;

    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.ser.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.ser.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.ser.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.ser.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.ser.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.ser.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.ser.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.ser.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.ser.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.ser.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.ser.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.ser.serialize_char(v)
    }

    // TODO quote strings with spaces
    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.formatter.write_key(&mut self.ser.writer, v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.ser.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<()> {
        self.ser.serialize_none()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ser.serialize_some(value)
    }

    fn serialize_unit(self) -> Result<()> {
        self.ser.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        self.ser.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.ser
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ser.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        self.ser
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_tuple(
        self,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::SerdeError("explode!".to_string()))
    }
}

#[derive(Debug)]
pub struct Serializer<W, F = DefaultFormatter> {
    /// The working string that things are serialized into.
    writer: W,
    formatter: F,
}

impl Serializer<Vec<u8>, DefaultFormatter> {
    fn new() -> Self {
        Self {
            writer: Vec::default(),
            formatter: DefaultFormatter::default(),
        }
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::Serializer for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.formatter.write_bool(&mut self.writer, v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.formatter.write_number(&mut self.writer, v as f64)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.formatter.write_number(&mut self.writer, v as f64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.formatter.write_number(&mut self.writer, v)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // TODO handle strings with escapes
    fn serialize_str(self, v: &str) -> Result<()> {
        self.formatter.begin_string(&mut self.writer)?;
        self.formatter.write_string_fragment(&mut self.writer, v)?;
        self.formatter.end_string(&mut self.writer)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;

        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }

        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.formatter.write_null(&mut self.writer)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        self.formatter.begin_dict(&mut self.writer)?;
        self.formatter.write_key(&mut self.writer, variant)?;
        value.serialize(&mut *self)?;
        self.formatter.write_newline(&mut self.writer)?;
        self.formatter.end_dict(&mut self.writer)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.formatter.begin_list(&mut self.writer)?;

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.formatter.begin_dict(&mut self.writer)?;
        self.formatter.write_key(&mut self.writer, variant)?;
        self.formatter.begin_list(&mut self.writer)?;

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.formatter.begin_dict(&mut self.writer)?;

        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.formatter.begin_dict(&mut self.writer)?;

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.formatter.begin_dict(&mut self.writer)?;
        self.formatter.write_key(&mut self.writer, variant)?;

        self.formatter.begin_dict(&mut self.writer)?;

        Ok(self)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeSeq for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.formatter.write_indent(&mut self.writer, None)?;
        value.serialize(&mut **self)?;
        self.formatter.write_newline(&mut self.writer)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_list(&mut self.writer)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeTuple for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeTupleStruct for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeTupleVariant for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_list(&mut self.writer)?;
        self.formatter.write_newline(&mut self.writer)?;
        self.formatter.end_dict(&mut self.writer)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeMap for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        key.serialize(KeySerializer::new(*self))
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        self.formatter.write_newline(&mut self.writer)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        self.serialize_key(key)?;
        ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_dict(&mut self.writer)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.formatter.write_key(&mut self.writer, key)?;
        value.serialize(&mut **self)?;
        self.formatter.write_newline(&mut self.writer)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_dict(&mut self.writer)
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeStructVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.formatter.write_key(&mut self.writer, key)?;
        value.serialize(&mut **self)?;
        self.formatter.write_newline(&mut self.writer)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_dict(&mut self.writer)?;
        self.formatter.write_newline(&mut self.writer)?;
        self.formatter.end_dict(&mut self.writer)
    }
}

pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let mut serializer = Serializer::new();

    value.serialize(&mut serializer)?;

    // TODO Enum roots don't insert an ending newline so insert a newline manually for now
    if !serializer.writer.ends_with(b"\n") {
        serializer.writer.extend_from_slice(b"\n");
    }

    String::from_utf8(serializer.writer).map_err(|e| Error::SerdeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use std::collections::BTreeMap;

    use super::to_string;

    mod primitive_tests {
        use super::*;

        #[test]
        fn test_tuple() {
            let data = ("hello", "world", true);

            let output = to_string(&data).unwrap();

            assert_eq!(
                output,
                "\
[
    \"hello\"
    \"world\"
    true
]
"
            )
        }

        #[test]
        fn test_tuple_nested() {
            let data = (
                "hello",
                true,
                (false, 123.0),
                BTreeMap::<String, String>::new(),
            );

            let output = to_string(&data).unwrap();

            assert_eq!(
                output,
                "\
[
    \"hello\"
    true
    [
        false
        123.0
    ]
    {
    }
]
"
            )
        }

        #[test]
        fn test_vec() {
            let data = vec![true, false, true];

            let output = to_string(&data).unwrap();

            assert_eq!(
                output,
                "\
[
    true
    false
    true
]
"
            )
        }

        // NOTE BTreeMap is used for consistent ordering
        #[test]
        fn test_map() {
            let data = {
                let mut map = BTreeMap::new();
                map.insert("hello".to_string(), 123.0);
                map.insert("world".to_string(), 0.0);
                map
            };

            let output = to_string(&data).unwrap();

            assert_eq!(
                output,
                "\
hello 123.0
world 0.0
"
            )
        }

        #[test]
        fn test_map_nested() {
            let data = {
                let mut map = BTreeMap::new();
                map.insert("inner1", {
                    let mut map = BTreeMap::new();
                    map.insert("my-key", "my-value");

                    map
                });
                map.insert("inner2", {
                    let mut map = BTreeMap::new();
                    map.insert("other", "val");

                    map
                });

                map
            };

            let output = to_string(&data).unwrap();

            assert_eq!(
                output,
                "\
inner1 {
    my-key \"my-value\"
}
inner2 {
    other \"val\"
}
"
            )
        }
    }

    #[cfg(test)]
    mod struct_tests {
        use super::*;

        #[test]
        fn test_struct_unit() {
            #[derive(Serialize)]
            struct TestStruct;

            let output = to_string(&TestStruct).unwrap();

            assert_eq!(output, "null\n");
        }

        #[test]
        fn test_struct_empty() {
            #[derive(Serialize)]
            struct TestStruct {}

            let output = to_string(&TestStruct {}).unwrap();

            assert_eq!(output, "\n");
        }

        #[test]
        fn test_struct_newtype_string() {
            #[derive(Serialize)]
            struct TestStruct(String);

            let output = to_string(&TestStruct("hello world".to_string())).unwrap();

            assert_eq!(output, "\"hello world\"\n");
        }

        #[test]
        fn test_struct_newtype_number() {
            #[derive(Serialize)]
            struct TestStruct(f64);

            let output = to_string(&TestStruct(10.0)).unwrap();

            assert_eq!(output, "10.0\n");
        }

        #[test]
        fn test_struct_newtype_inner_newtype() {
            #[derive(Serialize)]
            struct TestStruct(Inner);
            #[derive(Serialize)]
            struct Inner(i32);

            let output = to_string(&TestStruct(Inner(100))).unwrap();

            assert_eq!(output, "100.0\n");
        }

        #[test]
        fn test_struct_newtype_inner_struct() {
            #[derive(Serialize)]
            struct TestStruct(Inner);
            #[derive(Serialize)]
            struct Inner {
                string: String,
                boolean: bool,
            }

            let output = to_string(&TestStruct(Inner {
                string: "hello".to_string(),
                boolean: true,
            }))
            .unwrap();

            assert_eq!(
                output,
                "\
string \"hello\"
boolean true
"
            )
        }

        #[test]
        fn test_struct_inner_newtype_struct() {
            #[derive(Serialize)]
            struct TestStruct {
                inner: Inner,
            }
            #[derive(Serialize)]
            struct Inner(bool);

            let output = to_string(&TestStruct { inner: Inner(true) }).unwrap();

            assert_eq!(output, "inner true\n");
        }

        #[test]
        fn test_struct_nested_newtypes() {
            #[derive(Serialize)]
            struct TestStruct(Inner);
            #[derive(Serialize)]
            struct Inner(InnerInner);
            #[derive(Serialize)]
            struct InnerInner(bool);

            let output = to_string(&TestStruct(Inner(InnerInner(false)))).unwrap();

            assert_eq!(output, "false\n");
        }

        #[test]
        fn test_struct_tuple() {
            #[derive(Serialize)]
            struct TestStruct(i32, bool);

            let output = to_string(&TestStruct(10, false)).unwrap();

            assert_eq!(
                output,
                "\
[
    10.0
    false
]
"
            );
        }

        #[test]
        fn test_struct_with_unit() {
            #[derive(Serialize)]
            struct Inner;
            #[derive(Serialize)]
            struct TestStruct {
                inner: Inner,
            }

            let output = to_string(&TestStruct { inner: Inner }).unwrap();

            assert_eq!(output, "inner null\n");
        }

        #[test]
        fn test_struct_flat() {
            #[derive(Serialize)]
            struct TestStruct {
                boolean: bool,
                number: f64,
                int_number: i64,
                string: String,
                unit: Option<()>,
            }

            let output = to_string(&TestStruct {
                boolean: true,
                number: 10.0,
                int_number: 100,
                string: "hello, world!".to_string(),
                unit: None,
            })
            .unwrap();

            assert_eq!(
                output,
                "\
boolean true
number 10.0
int_number 100.0
string \"hello, world!\"
unit null
"
            )
        }

        #[test]
        fn test_struct_nested() {
            #[derive(Serialize)]
            struct Inner {
                num: f64,
                vec: Vec<i32>,
            }

            #[derive(Serialize)]
            struct TestStruct {
                boolean: bool,
                number: f64,
                int_number: i64,
                string: String,
                inner: Inner,
            }

            let test_struct = TestStruct {
                boolean: true,
                number: 10.0,
                int_number: 2,
                string: "hello world!".to_string(),
                inner: Inner {
                    num: 10.1,
                    vec: vec![1, 2, 3],
                },
            };
            let output = to_string(&test_struct).unwrap();

            assert_eq!(
                output,
                "\
boolean true
number 10.0
int_number 2.0
string \"hello world!\"
inner {
    num 10.1
    vec [
        1.0
        2.0
        3.0
    ]
}
"
            );
        }

        #[test]
        fn test_struct_nested_map() {
            #[derive(Serialize)]
            struct Inner {
                my_int: i32,
                my_float: f32,
            }

            #[derive(Serialize)]
            struct TestStruct {
                map: BTreeMap<String, String>, // NOTE: HashMap does not guarantee order while BTreeMap does
                array: Vec<i32>,
                inner: Inner,
            }

            let test_struct = TestStruct {
                map: {
                    let mut m = BTreeMap::new();
                    m.insert("hello".to_string(), "world".to_string());
                    m.insert("goodbye".to_string(), "bleh".to_string());
                    m
                },
                array: vec![1, 2, 3],
                inner: Inner {
                    my_int: 100,
                    my_float: 50.0,
                },
            };

            let output = to_string(&test_struct).unwrap();

            assert_eq!(
                output,
                "\
map {
    goodbye \"bleh\"
    hello \"world\"
}
array [
    1.0
    2.0
    3.0
]
inner {
    my_int 100.0
    my_float 50.0
}
"
            );
        }
    }

    mod enum_tests {
        use super::*;

        #[test]
        fn test_enum_unit() {
            #[derive(Serialize)]
            enum TestEnum {
                Unit,
            }

            let output = to_string(&TestEnum::Unit).unwrap();

            assert_eq!(output, "\"Unit\"\n");
        }

        #[test]
        fn test_enum_variant() {
            #[derive(Serialize)]
            enum TestEnum {
                Variant(i32),
            }

            let output = to_string(&TestEnum::Variant(10)).unwrap();

            assert_eq!(output, "Variant 10.0\n");
        }

        #[test]
        fn test_enum_variant_multi() {
            #[derive(Serialize)]
            enum TestEnum {
                MultiVariant(i32, bool),
            }

            let output = to_string(&TestEnum::MultiVariant(100, false)).unwrap();

            assert_eq!(
                output,
                "\
MultiVariant [
    100.0
    false
]
"
            )
        }

        #[test]
        fn test_enum_nested_enum_newtype() {
            #[derive(Serialize)]
            enum TestEnum {
                Inner(Inner),
            }

            #[derive(Serialize)]
            enum Inner {
                String(String),
            }

            let output = to_string(&TestEnum::Inner(Inner::String("hello".to_string()))).unwrap();

            assert_eq!(
                output,
                "\
Inner {
    String \"hello\"
}
"
            )
        }

        #[test]
        fn test_enum_nested_enum_tuple() {
            #[derive(Serialize)]
            enum TestEnum {
                Inner(Inner),
            }

            #[derive(Serialize)]
            enum Inner {
                Tuple(String, bool),
            }

            let output =
                to_string(&TestEnum::Inner(Inner::Tuple("hello".to_string(), true))).unwrap();

            assert_eq!(
                output,
                "\
Inner {
    Tuple [
        \"hello\"
        true
    ]
}
"
            )
        }

        #[test]
        fn test_enum_nested_enum_struct() {
            #[derive(Serialize)]
            enum TestEnum {
                Inner(Inner),
            }

            #[derive(Serialize)]
            enum Inner {
                Struct { string: String, boolean: bool },
            }

            let output = to_string(&TestEnum::Inner(Inner::Struct {
                string: "hello".to_string(),
                boolean: true,
            }))
            .unwrap();

            assert_eq!(
                output,
                "\
Inner {
    Struct {
        string \"hello\"
        boolean true
    }
}
"
            )
        }

        #[test]
        fn test_enum_variant_multi_nested() {
            #[derive(Serialize)]
            enum TestEnum {
                MultiVariant(i32, Inner),
            }

            #[derive(Serialize)]
            enum Inner {
                Variant(InnerInner, InnerInner),
            }

            #[derive(Serialize)]
            enum InnerInner {
                Unit,
                Struct { string: String, boolean: bool },
            }

            let output = to_string(&TestEnum::MultiVariant(
                100,
                Inner::Variant(
                    InnerInner::Unit,
                    InnerInner::Struct {
                        string: "hello".to_string(),
                        boolean: true,
                    },
                ),
            ))
            .unwrap();

            assert_eq!(
                output,
                "\
MultiVariant [
    100.0
    {
        Variant [
            \"Unit\"
            {
                Struct {
                    string \"hello\"
                    boolean true
                }
            }
        ]
    }
]
"
            )
        }

        #[test]
        fn test_enum_tuple() {
            #[derive(Serialize)]
            enum TestEnum {
                Tuple((i32, bool)),
            }

            let output = to_string(&TestEnum::Tuple((100, false))).unwrap();

            assert_eq!(
                output,
                "\
Tuple [
    100.0
    false
]
"
            )
        }

        #[test]
        fn test_map_with_enum() {
            #[derive(Serialize)]
            enum TupleEnum {
                Num(i32),
                Tuple((i32, i32)),
            }

            let mut map = BTreeMap::new();
            map.insert("val1", TupleEnum::Num(10));
            map.insert("val2", TupleEnum::Num(20));
            map.insert("val3", TupleEnum::Tuple((10, 20)));

            let output = to_string(&map).unwrap();

            assert_eq!(
                output,
                "\
val1 {
    Num 10.0
}
val2 {
    Num 20.0
}
val3 {
    Tuple [
        10.0
        20.0
    ]
}
"
            );
        }
    }

    #[test]
    fn test_struct_and_enum() {
        {
            #[derive(Serialize)]
            struct TestStruct {
                inner: InnerTestStruct,
                enum_unit: TestEnum,
                enum_var_prim: TestEnum,
                enum_var_stru: TestEnum,
                enum_struct: TestEnum,
            }

            #[derive(Serialize)]
            struct InnerTestStruct {
                boolean: bool,
                number: f64,
                string: String,
            }

            #[derive(Serialize)]
            enum TestEnum {
                Unit,
                TupleVariantPrimitive(i32),
                TupleVariantStruct(InnerTestStruct),
                Struct { inner: InnerTestStruct },
            }

            let test_struct = TestStruct {
                inner: InnerTestStruct {
                    boolean: true,
                    number: 10.0,
                    string: "inner".to_string(),
                },
                enum_unit: TestEnum::Unit,
                enum_var_prim: TestEnum::TupleVariantPrimitive(22),
                enum_var_stru: TestEnum::TupleVariantStruct(InnerTestStruct {
                    boolean: false,
                    number: 321.0,
                    string: "enum_var_stru".to_string(),
                }),
                enum_struct: TestEnum::Struct {
                    inner: InnerTestStruct {
                        boolean: true,
                        number: 0.0,
                        string: "enum_struct".to_string(),
                    },
                },
            };

            let output = to_string(&test_struct).unwrap();

            assert_eq!(
                output,
                "\
inner {
    boolean true
    number 10.0
    string \"inner\"
}
enum_unit \"Unit\"
enum_var_prim {
    TupleVariantPrimitive 22.0
}
enum_var_stru {
    TupleVariantStruct {
        boolean false
        number 321.0
        string \"enum_var_stru\"
    }
}
enum_struct {
    Struct {
        inner {
            boolean true
            number 0.0
            string \"enum_struct\"
        }
    }
}
"
            );
        }
    }
}
