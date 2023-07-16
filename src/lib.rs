#[cfg(feature = "standalone")]
pub mod cli;

pub mod de;
pub mod ser;

mod error;
pub use error::{Error, Result};

pub mod parser;
pub use parser::TotValue;
