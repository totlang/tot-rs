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

    fn write_space<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b" ").map_err(Error::Io)
    }

    fn write_newline<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"\n").map_err(Error::Io)
    }

    fn write_indent<W: ?Sized + std::io::Write>(
        &mut self,
        writer: &mut W,
        amount: Option<usize>,
    ) -> Result<()> {
        for _ in 1..amount.unwrap_or(self.get_indent()) {
            writer.write_all(INDENT.as_bytes()).map_err(Error::Io)?;
        }

        Ok(())
    }

    #[inline]
    fn write_null<W: ?Sized + std::io::Write>(&mut self, writer: &mut W) -> Result<()> {
        writer.write_all(b"()").map_err(Error::Io)
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

#[derive(Debug, Default)]
pub struct DefaultFormatter {
    indents: usize,
    was_newline: bool,
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
}

pub struct CompactFormatter;

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

    // TODO the serde tutorial mentions that using the itoa crate will be faster
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.formatter.write_number(&mut self.writer, v as f64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v.into())
    }

    // TODO the serde tutorial mentions that using the itoa crate will be faster
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
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.formatter.begin_dict(&mut self.writer)?;
        self.formatter.write_key(&mut self.writer, variant)?;
        self.formatter.begin_list(&mut self.writer)?;

        self.serialize_seq(Some(len))
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        self.formatter.begin_dict(&mut self.writer)?;

        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        self.formatter.begin_dict(&mut self.writer)?;
        self.formatter.write_key(&mut self.writer, variant)?;

        self.serialize_map(Some(len))
    }
}

impl<'a, W: std::io::Write, F: Formatter> ser::SerializeSeq for &'a mut Serializer<W, F> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
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

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
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

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
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

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_list(&mut self.writer)?;
        self.formatter.end_dict(&mut self.writer)
    }
}

// TODO need to use custom object so we can write keys as literal strings
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

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
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

pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let mut serializer = Serializer {
        writer: Vec::new(),
        formatter: DefaultFormatter::default(),
    };

    value.serialize(&mut serializer)?;

    String::from_utf8(serializer.writer).map_err(|e| Error::SerdeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[test]
    fn test_struct() {
        {
            #[derive(Serialize)]
            struct Inner {
                num: f64,
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
                inner: Inner { num: 10.1 },
            };
            let output = super::to_string(&test_struct).unwrap();

            assert_eq!(output, "boolean true\nnumber 10.0\nint_number 2.0\nstring \"hello world!\"\ninner {\n    num 10.1\n}\n");
        }
        {
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

            let output = super::to_string(&test_struct).unwrap();

            assert_eq!(output, "map {\n    goodbye \"bleh\"\n    hello \"world\"\n}\narray [\n    1.0\n    2.0\n    3.0\n]\ninner {\n    my_int 100.0\n    my_float 50.0\n}\n");
        }
    }

    #[test]
    fn test_enum() {
        // {
        //     #[derive(Serialize)]
        //     enum TestEnum {
        //         Unit,
        //         Tuple(i32),
        //         Struct { string: String, list: Vec<bool> },
        //     }

        //     let test_enum = TestEnum::Unit;
        //     let output = super::to_string(&test_enum).unwrap();

        //     assert_eq!(output, "\"Unit\"");

        //     let test_enum = TestEnum::Tuple(10);
        //     let output = super::to_string(&test_enum).unwrap();

        //     assert_eq!(output, "Tuple 10.0\n");

        //     let test_enum = TestEnum::Struct {
        //         string: "Hello".to_string(),
        //         list: vec![true, false],
        //     };
        //     let output = super::to_string(&test_enum).unwrap();

        //     assert_eq!(output, "Struct {\n    string \"Hello\"\n    list [\n        true\n        false\n    ]\n}\n");
        // }
        {
            #[derive(Serialize)]
            enum TupleEnum {
                Num(i32),
                Tuple((i32, i32)),
            }

            let mut map = BTreeMap::new();
            map.insert("val1", TupleEnum::Num(10));
            map.insert("val2", TupleEnum::Num(20));

            let output = super::to_string(&map).unwrap();

            assert_eq!(output, "val1");
        }
    }
}
