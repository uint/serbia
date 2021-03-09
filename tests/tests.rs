// use serbia::serbia;
// use serde::Serialize;

// #[test]
// fn regular_struct() {
//     #[serbia]
//     #[derive(Debug, Serialize, Deserialize, PartialEq)]
//     struct S {
//         arr_a: [u8; 300],
//         arr_b: [u8; 42],
//         arr_small: [u8; 8],
//     }

//     let original = S {
//         arr_a: [0; 300],
//         arr_b: [0; 42],
//         arr_small: [0; 8],
//     };

//     let serialized = serde_yaml::to_string(&original).unwrap();
//     let deserialized = serde_yaml::from_str(&serialized).unwrap();

//     assert_eq!(original, deserialized);
// }
