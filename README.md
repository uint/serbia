# serbia
Serde big arrays. An attribute macro to make (de)serializing big arrays painless, following a [design proposed by dtolnay](https://github.com/dtolnay/request-for-implementation/issues/17).

## Why?
I saw the idea in [request-for-implementation](https://github.com/dtolnay/request-for-implementation). Then I came up with the name.

The name was too good. I had to do it. Don't judge me.

Also: Serbia has some tasty food.

## Status
Early research and development. Hold your breath though!

## Usage
Just slap `#[serbia]` on top of your struct definition.

```rust
use serbia::serbia;

#[serbia]
#[derive(Serialize, Deserialize)]
struct S {
    arr_a: [u8; 300],
    arr_b: [u8; 42],
    arr_small: [u8; 8],
}
```