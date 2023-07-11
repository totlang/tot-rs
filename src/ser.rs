use serde::{ser, Serialize};

use crate::error::{Error, Result};

// TODO might need to account for indentation depth
const OPEN_CURLY: &str = " {\n";
const CLOSE_CURLY: &str = "\n}";
const OPEN_SQUARE: &str = " [\n";
const CLOSE_SQUARE: &str = "\n]";

#[derive(Default)]
pub struct Serializer {
    /// The working string that things are serialized into.
    output: String,
    /// The number of indents. Used for determining how to format closing brackets.
    indents: u32,
}

impl Serializer {}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    // TODO the serde tutorial mentions that using the itoa crate will be faster
    fn serialize_i64(self, v: i64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    // TODO the serde tutorial mentions that using the itoa crate will be faster
    fn serialize_u64(self, v: u64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    // TODO the serde tutorial mentions that using the itoa crate will be faster
    fn serialize_f64(self, v: f64) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    // TODO handle strings with quotes
    fn serialize_str(self, v: &str) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> std::result::Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;

        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }

        seq.end()
    }

    fn serialize_none(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        self.output += "()";
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> std::result::Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> std::result::Result<Self::Ok, Self::Error>
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
    ) -> std::result::Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        // self.output += OPEN_CURLY;
        // self.indents += 1;
        bracket(self, Bracket::OpenCurly);
        variant.serialize(&mut *self)?;
        self.output += " ";
        value.serialize(&mut *self)?;
        // self.indents -= 1;
        // self.output += CLOSE_CURLY;
        bracket(self, Bracket::CloseCurly);

        Ok(())
    }

    fn serialize_seq(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        // self.output += OPEN_SQUARE;
        // self.indents += 1;
        bracket(self, Bracket::OpenSquare);

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        // self.output += OPEN_CURLY;
        // self.indents += 1;
        bracket(self, Bracket::OpenCurly);
        variant.serialize(&mut *self)?;
        // self.output += OPEN_SQUARE;
        // self.indents += 1;
        bracket(self, Bracket::OpenSquare);

        Ok(self)
    }

    fn serialize_map(
        self,
        _len: Option<usize>,
    ) -> std::result::Result<Self::SerializeMap, Self::Error> {
        // if self.indents > 0 {
        //     self.output += OPEN_CURLY;
        // }
        // self.indents += 1;
        bracket(self, Bracket::OpenCurly);

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
        _len: usize,
    ) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        // self.output += OPEN_CURLY;
        // self.indents += 1;
        bracket(self, Bracket::OpenCurly);
        variant.serialize(&mut *self)?;
        // self.output += OPEN_CURLY;
        // self.indents += 1;
        bracket(self, Bracket::OpenCurly);

        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // self.output += CLOSE_SQUARE;
        bracket(self, Bracket::CloseSquare);

        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // self.output += CLOSE_SQUARE;
        bracket(self, Bracket::CloseSquare);

        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // self.output += CLOSE_SQUARE;
        bracket(self, Bracket::CloseSquare);

        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // self.output += CLOSE_SQUARE;
        // self.indents -= 1;
        // self.output += CLOSE_CURLY;
        bracket(self, Bracket::CloseSquare);
        bracket(self, Bracket::CloseCurly);

        Ok(())
    }
}

// TODO need to use custom object so we can write keys as literal strings
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // if self.indents > 0 {
        //     self.output += CLOSE_CURLY;
        // }
        bracket(self, Bracket::CloseCurly);

        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
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
        self.output += " ";
        key.serialize(&mut **self)?;
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        // self.indents -= 1;
        // if self.indents > 0 {
        //     self.output += CLOSE_CURLY;
        // }
        bracket(self, Bracket::CloseCurly);

        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
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
        self.output += " ";
        key.serialize(&mut **self)?;
        self.output += " ";
        value.serialize(&mut **self)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        bracket(self, Bracket::CloseCurly);
        // self.indents -= 1;
        // self.output += CLOSE_CURLY;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Bracket {
    OpenCurly,
    CloseCurly,

    OpenSquare,
    CloseSquare,
}

fn bracket(s: &mut Serializer, b: Bracket) {
    match b {
        Bracket::OpenCurly => {
            if s.indents > 0 {
                s.output += OPEN_CURLY;
            }
            s.indents += 1;
        }
        Bracket::CloseCurly => {
            s.indents -= 1;
            if s.indents > 0 {
                s.output += CLOSE_CURLY;
            }
        }
        Bracket::OpenSquare => {
            if s.indents > 0 {
                s.output += OPEN_SQUARE
            }
            s.indents += 1;
        }
        Bracket::CloseSquare => {
            s.indents -= 1;
            if s.indents > 0 {
                s.output += CLOSE_SQUARE;
            }
        }
    }
}

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    let mut serializer = Serializer::default();

    value.serialize(&mut serializer)?;

    Ok(serializer.output)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_struct() {
        #[derive(serde::Serialize)]
        struct Inner {
            num: f64,
        }

        #[derive(serde::Serialize)]
        struct TestStruct {
            boolean: bool,
            number: f64,
            string: String,
            inner: Inner,
        }

        let test_struct = TestStruct {
            boolean: true,
            number: 10.0,
            string: "hello world!".to_string(),
            inner: Inner { num: 10.1 },
        };
        let output = super::to_string(&test_struct).unwrap();

        assert_eq!(output, "asdf");
    }
}
