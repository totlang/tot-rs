/*!
A Rust implementation of [Tot](https://github.com/totlang/tot).

Tot is a configuration language meant to be edited by hand.

# Features

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

# Example

```
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

fn main() {
    let person = Person {
        name: "youwin".to_string(),
        age: 100
    };

    let output = tot::to_string(&person).unwrap();

    assert_eq!("\
name \"youwin\"
age 100.0
", output);

    let person = tot::from_str::<Person>(output.as_str()).unwrap();

    assert_eq!(person.name, "youwin");
    assert_eq!(person.age, 100);
}
```

*/

pub mod de;
pub use de::from_str;
pub mod ser;
pub use ser::to_string;

mod error;
pub use error::{Error, Result};

pub mod parser;
pub use parser::TotValue;
