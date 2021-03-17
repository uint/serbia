use serbia::serbia;
use serde::{Deserialize, Serialize};

fn main() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S([u8; 300], [u8; 42], [u8; 8]);

    let original = S([0; 300], [0; 42], [0; 8]);

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}