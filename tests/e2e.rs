use serde::{Deserialize, Serialize};
use tot::{from_str, to_string};

#[test]
fn test_simple_struct() {
    #[derive(Serialize, Deserialize)]
    struct Data {
        boolean: bool,
        integer: i32,
        string: String,
    }

    let data = Data {
        boolean: true,
        integer: 22,
        string: "hello world".to_string(),
    };

    let output = to_string(&data).unwrap();
    assert_eq!(
        output,
        "\
boolean true
integer 22.0
string \"hello world\"
"
    );

    let output = from_str::<Data>(output.as_str()).unwrap();
    assert_eq!(output.boolean, data.boolean);
    assert_eq!(output.integer, data.integer);
    assert_eq!(output.string, data.string);
}

#[test]
fn test_nested_struct() {
    #[derive(Serialize, Deserialize)]
    struct Data {
        boolean: bool,
        fields: Fields,
    }

    #[derive(Serialize, Deserialize)]
    struct Fields {
        key1: String,
        key2: String,
        key3: String,
    }

    let data = Data {
        boolean: true,
        fields: Fields {
            key1: "hello".to_string(),
            key2: "world".to_string(),
            key3: "goodbye".to_string(),
        },
    };

    let output = to_string(&data).unwrap();
    assert_eq!(
        output,
        "\
boolean true
fields {
    key1 \"hello\"
    key2 \"world\"
    key3 \"goodbye\"
}
"
    );

    let output = from_str::<Data>(output.as_str()).unwrap();
    assert_eq!(output.boolean, data.boolean);
    assert_eq!(output.fields.key1, data.fields.key1);
    assert_eq!(output.fields.key2, data.fields.key2);
    assert_eq!(output.fields.key3, data.fields.key3);
}
