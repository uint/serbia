use serbia::serbia;
use serde::{Deserialize, Serialize};

fn main() {
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