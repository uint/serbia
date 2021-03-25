use serbia::serbia;
use serde::{ser::SerializeTuple, Deserialize, Serialize};

#[test]
fn serialize_deserialize_with() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serde(serialize_with = "ser", deserialize_with = "de")]
        big_arr: [u8; 42],
    }

    let original = S { big_arr: [0; 42] };

    let expected = S { big_arr: [5; 42] };

    fn ser<S>(array: &[u8; 42], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_tuple(42)?;
        for _ in array {
            // This is purposely wrong so that we can later test if this serializer
            // was in fact used.
            seq.serialize_element(&5)?;
        }
        seq.end()
    }

    fn de<'de, D>(deserializer: D) -> core::result::Result<[u8; 42], D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = [u8; 42];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(std::concat!("an array"))
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                unsafe {
                    let mut arr: Self::Value = std::mem::MaybeUninit::uninit().assume_init();

                    for (i, v) in arr.iter_mut().enumerate() {
                        *v = match seq.next_element()? {
                            Some(val) => val,
                            None => return Err(serde::de::Error::invalid_length(i, &self)),
                        };
                    }

                    Ok(arr)
                }
            }
        }

        deserializer.deserialize_tuple(42, Visitor)
    }

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_ne!(original, deserialized);
    assert_eq!(deserialized, expected);
}

#[test]
fn skip_field() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serde(skip, default = "def")]
        arr_a: [u8; 42],
    }

    fn def() -> [u8; 42] {
        [3; 42]
    }

    let original = S { arr_a: [0; 42] };

    let expected = S { arr_a: [3; 42] };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_ne!(original, deserialized);
    assert_eq!(expected, deserialized);
}

#[test]
fn skip_serializing() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serde(skip_serializing)]
        arr_a: [u8; 42],
    }

    let original = S { arr_a: [2; 42] };

    let expected = S { arr_a: [0; 42] };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized: Result<S, _> = serde_yaml::from_str(&serialized);

    assert!(deserialized.is_err());

    let deserialized = serde_yaml::from_str(r#"
        arr_a: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    "#).unwrap();

    assert_eq!(expected, deserialized);
}

#[test]
fn skip_serializing_if_true() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serde(skip_serializing_if = "t")]
        arr_a: [u8; 42],
    }

    fn t<T>(_: T) -> bool {
        true
    }

    let original = S { arr_a: [2; 42] };

    let expected = S { arr_a: [0; 42] };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized: Result<S, _> = serde_yaml::from_str(&serialized);

    assert!(deserialized.is_err());

    let deserialized = serde_yaml::from_str(r#"
        arr_a: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    "#).unwrap();

    assert_eq!(expected, deserialized);
}

#[test]
fn skip_serializing_if_false() {
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        #[serde(skip_serializing_if = "f")]
        arr_a: [u8; 42],
    }

    fn f<T>(_: T) -> bool {
        false
    }

    let original = S { arr_a: [2; 42] };

    let serialized = serde_yaml::to_string(&original).unwrap();
    let deserialized = serde_yaml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}
