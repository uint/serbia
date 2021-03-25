use serbia::serbia;
use serde::{Deserialize, Serialize};

#[test]
fn type_alias() {
    const BUFSIZE: usize = 300;
    type BigArray = [i32; BUFSIZE];

    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serbia(bufsize = "BUFSIZE")]
        arr_a: BigArray,
        arr_b: [u8; 42],
        arr_small: [u8; 8],
    }

    let original = S {
        arr_a: [0; BUFSIZE],
        arr_b: [0; 42],
        arr_small: [0; 8],
    };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn skip_field() {
    const BUFSIZE: usize = 24;

    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serbia(skip = true)]
        arr_a: [u8; BUFSIZE],
        arr_b: [u8; 42],
        arr_small: [u8; 8],
    }

    let original = S {
        arr_a: [0; BUFSIZE],
        arr_b: [0; 42],
        arr_small: [0; 8],
    };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}
