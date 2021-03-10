use serbia::serbia;
use serde::{Deserialize, Serialize};

#[test]
fn regular_struct_roundtrip() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        arr_a: [u8; 300],
        arr_b: [u8; 42],
        arr_small: [u8; 8],
    }

    let original = S {
        arr_a: [0; 300],
        arr_b: [0; 42],
        arr_small: [0; 8],
    };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn tuple_struct_roundtrip() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S([u8; 300], [u8; 42], [u8; 8]);

    let original = S([0; 300], [0; 42], [0; 8]);

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}
