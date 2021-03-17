use serbia::serbia;
use serde::{Deserialize, Serialize};

const BUFSIZE: usize = 300;

fn main() {
    // We do not handle constants unless they field is explicitly marked with
    // serde_bufsize.

    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        arr_a: [u8; BUFSIZE],
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