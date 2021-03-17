use serbia::serbia;
use serde::{Deserialize, Serialize};

fn main() {
    const BUFSIZE: usize = 300;
    type BigArray = [i32; BUFSIZE];

    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serbia_bufsize(BUFSIZE)]
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