use std::convert::TryInto;

use serbia::serbia;
use serde::{Deserialize, Serialize};

/// This one was in response to https://github.com/uint/serbia/issues/1
#[test]
fn mem_safety_regression_test() {
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

#[test]
fn deserializing_failure_mem_leak() {
    use lazy_static::lazy_static;
    use std::sync::Arc;

    // We create some ref-counted data so we can later check that some of its owners
    // were dropped.
    lazy_static! {
        static ref RC_STRING: Arc<String> = Arc::new("foo".to_string());
    }

    // Foo will create a strong ref to RC_STRING when deserialized into.
    struct Foo(Arc<String>);

    impl<'de> Deserialize<'de> for Foo {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            d.deserialize_str(SimpleVisitor)?;
            Ok(Foo(Arc::clone(&RC_STRING)))
        }
    }

    // A dummy visitor. It's not actually used to deserialize the strings,
    // just to consume and skip them.
    struct SimpleVisitor;

    impl<'de> serde::de::Visitor<'de> for SimpleVisitor {
        type Value = Foo;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(std::concat!("a string"))
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> {
            Ok(Foo(Arc::new(s.to_string())))
        }
    }

    #[serbia]
    #[derive(Deserialize)]
    struct S {
        #[serbia(bufsize = 5)]
        _arr_big: [Foo; 5],
    }

    let json = serde_json::json!({
        "arr_big": ["asd", "asd", "asd"]
    })
    .to_string();

    // This deserialization attempt should fail since there are only 3 fields
    // provided in the JSON.
    let faulty_struct: Result<S, _> = serde_json::from_str(&json);
    assert!(faulty_struct.is_err());

    // At this point, the strong refs created during the deserialization
    // attempt should have been dropped, so the strong count should be back to one.
    // If it's not, we have a memory leak.
    assert_eq!(Arc::strong_count(&RC_STRING), 1);
}
