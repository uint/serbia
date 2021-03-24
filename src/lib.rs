//! This crate provides the [serbia](macro@self::serbia) macro.

extern crate proc_macro;

mod item;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, parse_quote};

use crate::item::Item;

fn render_serialize_fn(fn_ident: &Ident, len: impl ToTokens) -> TokenStream {
    quote! {
        fn #fn_ident<E, S>(array: &[E; #len], serializer: S) -> core::result::Result<S::Ok, S::Error>
        where
            E: serde::Serialize,
            S: serde::Serializer,
        {
            use serde::ser::SerializeTuple;

            let mut seq = serializer.serialize_tuple(#len)?;
            for e in array {
                seq.serialize_element(e)?;
            }
            seq.end()
        }
    }
}

fn render_deserialize_fn(fn_ident: &Ident, len: impl ToTokens) -> TokenStream {
    quote! {
        fn #fn_ident<'de, E, D>(deserializer: D) -> core::result::Result<[E; #len], D::Error>
        where
            E: serde::Deserialize<'de>,
            D: serde::Deserializer<'de>,
        {
            struct ArrayVisitor<E> {
                _casper: std::marker::PhantomData<E>,
            }

            impl<E> ArrayVisitor<E> {
                fn new() -> Self {
                    Self {
                        _casper: std::marker::PhantomData,
                    }
                }
            }

            impl<'de, E> serde::de::Visitor<'de> for ArrayVisitor<E>
            where
                E: serde::Deserialize<'de>,
            {
                type Value = [E; #len];

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(std::concat!("an array"))
                }

                #[inline]
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    unsafe {
                        let mut arr: Self::Value =  std::mem::MaybeUninit::uninit().assume_init();

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

            deserializer.deserialize_tuple(#len, ArrayVisitor::new())
        }
    }
}

/// An attribute macro that enables (de)serializing arrays of length larger than 32 with [Serde](serde).
///
/// Simply slap it on top of your struct or enum, before the [Serialize](serde::Serialize)/[Deserialize](serde::Deserialize) derive.
///
/// # Basic usage
/// ```edition2018
/// use serbia::serbia;
/// use serde::{Deserialize, Serialize};
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     arr_big: [u8; 300],   // Custom (de)serializers will be generated for this.
///     arr_small: [u8; 8],   // Nothing done here - Serde handles arrays up to length 32 just fine.
///     s: String,
/// }
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// enum E {
///     ArrBig([u8; 300]),
///     ArrSmall([u8; 22]),
///     Mixed([u8; 8], [i32; 44], String),
/// }
/// ```
///
/// # Specifying buffer size
///
/// You can use the `#[serbia_bufsize( ... )]` attribute to set a buffer size for
/// a field. This can be useful for type aliases. Constants work.
///
/// ```rust
/// use serbia::serbia;
/// use serde::{Deserialize, Serialize};
///
/// const BUFSIZE: usize = 300;
/// type BigArray = [i32; BUFSIZE];
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     #[serbia(bufsize = "BUFSIZE")]
///     arr_a: BigArray,
///     foo: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn serbia(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Item);
    let context = input.context();

    let mut fn_defs = vec![];

    for (i, field) in input.big_array_fields().enumerate() {
        if context.serialize {
            let fn_ident = format_ident!("serbia_serialize_{}_arr_{}", context.type_name, i);
            let fn_name = fn_ident.to_string();

            field.field.attrs.push(parse_quote! {
                #[serde(serialize_with = #fn_name)]
            });

            fn_defs.push(render_serialize_fn(&fn_ident, &field.len));
        }
        if context.deserialize {
            let fn_ident = format_ident!("serbia_deserialize_{}_arr_{}", context.type_name, i);
            let fn_name = fn_ident.to_string();

            field.field.attrs.push(parse_quote! {
                #[serde(deserialize_with = #fn_name)]
            });

            fn_defs.push(render_deserialize_fn(&fn_ident, &field.len));
        }
    }

    let expanded = quote! {
        #input
        #(#fn_defs)*
    };

    proc_macro::TokenStream::from(expanded)
}
