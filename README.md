# serbia

[![docs.rs badge](https://docs.rs/serbia/badge.svg)](https://docs.rs/serbia/)
[![crates.io badge](https://img.shields.io/crates/v/serbia.svg)](https://crates.io/crates/serbia/)
[![Build Status](https://github.com/uint/serbia/workflows/serbia%20CI/badge.svg)](https://github.com/uint/serbia)

Serde big arrays. An attribute macro to make (de)serializing big arrays painless, following a [design proposed by dtolnay](https://github.com/dtolnay/request-for-implementation/issues/17).

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
    arr_a: [u8; 300],
    arr_b: [u8; 42],
    arr_small: [u8; 8],
}

#[serbia]
#[derive(Serialize, Deserialize)]
enum E {
    ArrBig([u8; 300]),
    ArrSmall([u8; 22]),
    Mixed([u8; 8], [i32; 44], String),
}
```

You can use the `#[serbia_bufsize( ... )]` attribute to set a buffer size for
a field. This can be useful for type aliases. Constants work here!

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