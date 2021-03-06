# Serbia

[![docs.rs badge](https://img.shields.io/docsrs/serbia?style=flat-square)](https://docs.rs/serbia/)
[![crates.io badge](https://img.shields.io/crates/v/serbia.svg?style=flat-square&logo=rust)](https://crates.io/crates/serbia/)
[![Build Status](https://img.shields.io/github/workflow/status/uint/serbia/serbia%20CI?logo=Github&style=flat-square)](https://github.com/uint/serbia)

**Ser**de **bi**g **a**rrays. An attribute macro to make (de)serializing big arrays painless, roughly following a [design proposed by David Tolnay](https://github.com/dtolnay/request-for-implementation/issues/17).

## Why?
I saw the idea in [request-for-implementation](https://github.com/dtolnay/request-for-implementation). Then I came up with the name.

The name was too good. I had to do it. Don't judge me.

Also: Serbia has some tasty food.

## But what is it for?
Serde only implements `Serialize`/`Deserialize` for arrays of length up to 32. This is due to Rust's current limitation - we can't be generic over array length, so
an arbitrary upper limit was chosen and implementations were generated only up to it.

The crate provides a macro that generates all the code you need to (de)serialize arrays bigger than that.

## Status
Under development, but functional. Let me know what's missing or broken!

## Usage
Just slap `#[serbia]` on top of your type definition. Structs and enums both work!

```rust
use serbia::serbia;

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    arr_big: [u8; 300],     // custom serialize/deserialize code generated here
    arr_small: [u8; 8],     // no custom code - this is handled by Serde fine
}

#[serbia]
#[derive(Serialize, Deserialize)]
enum E {
    ArrBig([u8; 300]),
    ArrSmall([u8; 22]),
    Mixed([u8; 8], [i32; 44], String),
}
```

If *Serbia* sees an array length given as a constant, it will generate custom
serialize/deserialize code by default, without inspecting whether the constant
is larger than 32 or not. This is a limitation of macros.

```rust
const BUFSIZE: usize = 22;

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    arr: [i32; BUFSIZE],   // custom serialize/deserialize code generated here
    foo: String,
}
```

### Skipping fields

If for some reason you don't want *Serbia* to generate custom serialize/deserialize
code for a field that it would normally handle, you can skip it.

```rust
const BUFSIZE: usize = 24;

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    #[serbia(skip)]
    arr_a: [u8; BUFSIZE],
    arr_b: [u8; 42],
    arr_small: [u8; 8],
}
```

It's possible to be more granular if needed for some reason.

```rust
const BUFSIZE: usize = 24;

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    #[serbia(skip_serializing, skip_deserializing)]
    arr_a: [u8; BUFSIZE],
    arr_b: [u8; 42],
    arr_small: [u8; 8],
}
```

### Manual array length

You can use the `#[serbia(bufsize = ... )]` option to set an array length for
a field. This can be useful to make type aliases work. Constants work here!

```rust
type BigArray = [i32; 300];

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    #[serbia(bufsize = 300)]
    arr_a: BigArray,
    foo: String,
}
```

```rust
const BUFSIZE: usize = 300;
type BigArray = [i32; BUFSIZE];

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    #[serbia(bufsize = "BUFSIZE")]
    arr_a: BigArray,
    foo: String,
}
```

### Interaction with Serde field attributes
*Serbia* detects when certain *Serde* field attributes are used and avoids
generating code that would cause a conflict, instead yielding to *Serde*.

```rust
    #[serbia]
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct S {
        big_arr: [u8; 40],    // serbia generates code for this
        #[serde(serialize_with="ser", deserialize_with="de")]
        bigger_arr: [u8; 42], // serbia ignores this in favor of the (de)serializers you provided
    }
```

*Serbia* is intended to play nice with *Serde* field attributes.
If there are problems, please create an issue or submit a PR!

## What doesn't work
Nested types.

```rust
#[serbia]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct S {
    big_arr: Option<[u8; 300]>,  // no code generated for this nested array
}
```

*Serbia* doesn't yet pick up on *Serde* variant attributes,
so there might be conflicts there. This can probably be worked around by using
`#[serbia(skip)]` on each field that *Serbia* would try to generate custom
(de)serialization code for.