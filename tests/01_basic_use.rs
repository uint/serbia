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

#[test]
fn enum_roundtrip() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum E {
        ArrBig([u8; 300]),
        ArrSmall([u8; 22]),
        Mixed([u8; 8], [i32; 44], String),
    }

    // 1
    let original = E::ArrBig([22; 300]);

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);

    // 2
    let original = E::ArrSmall([44; 22]);

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);

    // 3
    let original = E::Mixed([0; 8], [5; 44], "asd".to_string());

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn constant() {
    const BUFSIZE: usize = 300;

    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
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

// Nested types are not supported (yet?)
// #[test]
// fn nested_types() {
//     #[serbia]
//     #[derive(Debug, Serialize, Deserialize, PartialEq)]
//     struct S {
//         big_arr: Option<[u8; 300]>,
//     }

//     let original = S {
//         big_arr: Some([0; 300]),
//     };

//     let serialized = serde_yaml::to_string(&original).unwrap();
//     let deserialized = serde_yaml::from_str(&serialized).unwrap();

//     assert_eq!(original, deserialized);
// }
