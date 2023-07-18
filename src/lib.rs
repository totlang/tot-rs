/*!
A Rust implementation of [Tot](https://github.com/totlang/tot).

Tot is a configuration language meant to be edited by hand.

## Features

* Whitespace-based format that _does not_ require indentation
* Simple, limited syntax
* JSON-style objects and lists
* Reference values (WIP)
* File import (WIP)
* Non-Turing complete Lisp-style expressions (WIP)
* Compatible with:
    * JSON
    * YAML
    * TOML

*/

pub mod de;
pub use de::from_str;
pub mod ser;
pub use ser::to_string;

mod error;
pub use error::{Error, Result};

pub mod parser;
pub use parser::TotValue;
