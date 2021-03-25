use std::convert::TryInto;

use serbia::serbia;
use serde::{Deserialize, Serialize};

#[test]
fn mem_safety() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize)]
    struct S {
        arr_big: [String; 300],
    }

    let mut i = 0;
    let arr_big: [String; 300] = std::iter::from_fn(|| {
        i += 1;
        Some(i.to_string())
    })
    .take(300)
    .collect::<Vec<_>>()
    .try_into()
    .unwrap();
    serde_json::from_str::<S>(&serde_json::to_string(&S { arr_big }).unwrap()).unwrap();

    let j = serde_json::json!({
        "arr_big": []
    });
    drop(dbg!(serde_json::from_value::<S>(j)));
    println!("Reached");
}
