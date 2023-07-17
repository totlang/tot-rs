use std::{fmt::Display, num::TryFromIntError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("serde error: {0}")]
    SerdeError(String),
    #[error("parser error: {0}")]
    ParserError(crate::parser::Error),
    #[error("io error: {0}")]
    Io(std::io::Error),
}

// TODO stub
impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::SerdeError(msg.to_string())
    }
}

// TODO stub
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::SerdeError(msg.to_string())
    }
}

// TODO stub
impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self::SerdeError(value.to_string())
    }
}
