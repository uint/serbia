//! This crate provides the [serbia](macro@self::serbia) macro.

extern crate proc_macro;

mod item;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, parse_quote, Type, TypePath};

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
                    use std::mem::{self, MaybeUninit};

                    let mut arr: [MaybeUninit<E>; #len] = unsafe { MaybeUninit::uninit().assume_init() };

                    for (i, v) in arr.iter_mut().enumerate() {
                        *v = MaybeUninit::new(match seq.next_element()? {
                            Some(val) => val,
                            None => {
                                (&mut arr[0..i]).iter_mut().for_each(|elem| {
                                    // TODO This would be better with assume_init_drop nightly function
                                    // https://github.com/rust-lang/rust/issues/63567
                                    unsafe { std::ptr::drop_in_place(elem.as_mut_ptr()) };
                                });
                                return Err(serde::de::Error::invalid_length(i, &self));
                            }
                        });
                    }

                    Ok(unsafe { mem::transmute_copy::<_, [E; #len]>(&arr) })
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
/// # Usage
/// Just slap `#[serbia]` on top of your type definition. Structs and enums both work!
///
/// ```rust
/// use serbia::serbia;
/// use serde::{Serialize, Deserialize};
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     arr_big: [u8; 300],     // custom serialize/deserialize code generated here
///     arr_small: [u8; 8],     // no custom code - this is handled by Serde fine
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
/// If *Serbia* sees an array length given as a constant, it will generate custom
/// serialize/deserialize code by default, without inspecting whether the constant
/// is larger than 32 or not. This is a limitation of macros.
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
/// #
/// const BUFSIZE: usize = 22;
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     arr: [i32; BUFSIZE],   // custom serialize/deserialize code generated here
///     foo: String,
/// }
/// ```
///
/// ## Skipping fields
///
/// If for some reason you don't want *Serbia* to generate custom serialize/deserialize
/// code for a field that it would normally handle, you can skip it.
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
/// #
/// const BUFSIZE: usize = 24;
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     #[serbia(skip)]
///     arr_a: [u8; BUFSIZE],
///     arr_b: [u8; 42],
///     arr_small: [u8; 8],
/// }
/// ```
///
/// It's possible to be more granular if needed for some reason.
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
/// #
/// const BUFSIZE: usize = 24;
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     #[serbia(skip_serializing, skip_deserializing)]
///     arr_a: [u8; BUFSIZE],
///     arr_b: [u8; 42],
///     arr_small: [u8; 8],
/// }
/// ```
///
/// ## Manual array length
///
/// You can use the `#[serbia(bufsize = ... )]` option to set an array length for
/// a field. This can be useful to make type aliases work. Constants work here!
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
/// #
/// type BigArray = [i32; 300];
///
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     #[serbia(bufsize = 300)]
///     arr_a: BigArray,
///     foo: String,
/// }
/// ```
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
/// #
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
///
/// ## Interaction with Serde field attributes
/// *Serbia* detects when certain *Serde* field attributes are used and avoids
/// generating code that would cause a conflict, instead yielding to *Serde*.
///
/// ```rust
/// # use serbia::serbia;
/// # use serde::{ser::SerializeTuple, Serialize, Deserialize};
/// #
/// #[serbia]
/// #[derive(Serialize, Deserialize)]
/// struct S {
///     big_arr: [u8; 40],    // serbia generates code for this
///     #[serde(serialize_with="ser", deserialize_with="de")]
///     bigger_arr: [u8; 42], // serbia ignores this in favor of the (de)serializers you provided
/// }
/// # fn ser<S>(array: &[u8; 42], serializer: S) -> Result<S::Ok, S::Error>
/// # where
/// #     S: serde::Serializer,
/// # {
/// #     let mut seq = serializer.serialize_tuple(42)?;
/// #     for e in array {
/// #         seq.serialize_element(e)?;
/// #     }
/// #     seq.end()
/// # }
/// #
/// # fn de<'de, D>(deserializer: D) -> core::result::Result<[u8; 42], D::Error>
/// # where
/// #     D: serde::Deserializer<'de>,
/// # {
/// #     struct Visitor;
/// #
/// #     impl<'de> serde::de::Visitor<'de> for Visitor {
/// #         type Value = [u8; 42];
/// #
/// #         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
/// #             formatter.write_str(std::concat!("an array"))
/// #         }
/// #
/// #         #[inline]
/// #         fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
/// #         where
/// #             A: serde::de::SeqAccess<'de>,
/// #         {
/// #             unsafe {
/// #                 let mut arr: Self::Value = std::mem::MaybeUninit::uninit().assume_init();
/// #
/// #                 for (i, v) in arr.iter_mut().enumerate() {
/// #                     *v = match seq.next_element()? {
/// #                         Some(val) => val,
/// #                         None => return Err(serde::de::Error::invalid_length(i, &self)),
/// #                     };
/// #                 }
/// #
/// #                 Ok(arr)
/// #             }
/// #         }
/// #     }
/// #
/// #     deserializer.deserialize_tuple(42, Visitor)
/// # }
/// ```
///
/// *Serbia* is intended to play nice with *Serde* field attributes.
/// If there are problems, please create an issue or submit a PR!
///
/// # What doesn't work
/// Nested types.
///
/// ```compile_fail
/// # use serbia::serbia;
/// # use serde::{Serialize, Deserialize};
///
/// #[serbia]
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// struct S {
///     big_arr: Option<[u8; 300]>,  // no code generated for this nested array
/// }
/// ```
///
/// *Serbia* doesn't yet pick up on *Serde* variant attributes,
/// so there might be conflicts there. This can probably be worked around by using
/// `#[serbia(skip)]` on each field that *Serbia* would try to generate custom
/// (de)serialization code for.
#[proc_macro_attribute]
pub fn serbia(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Item);
    let context = input.context();

    let mut fn_defs = vec![];

    for (i, field) in input.big_array_fields().enumerate() {
        let mut generate_bounds_for = None;

        if let Some(Type::Path(TypePath { path: el_ty, .. })) = field.element_type {
            if let Some(el_ty) = el_ty.get_ident() {
                generate_bounds_for = context.generics.type_params().find(|ty| &ty.ident == el_ty);
            }
        }

        if context.serialize && field.serialize {
            let fn_ident = format_ident!("serbia_serialize_{}_arr_{}", context.type_name, i);
            let fn_name = fn_ident.to_string();

            field.field.attrs.push(parse_quote! {
                #[serde(serialize_with = #fn_name)]
            });
            if let Some(type_param) = generate_bounds_for {
                let bound: syn::TypeParam = parse_quote! {
                    #type_param: Serialize
                };
                let bound = bound.into_token_stream().to_string();

                field.field.attrs.push(parse_quote! {
                    #[serde(bound(serialize = #bound))]
                });
            }

            fn_defs.push(render_serialize_fn(&fn_ident, &field.len));
        }
        if context.deserialize && field.deserialize {
            let fn_ident = format_ident!("serbia_deserialize_{}_arr_{}", context.type_name, i);
            let fn_name = fn_ident.to_string();

            field.field.attrs.push(parse_quote! {
                #[serde(deserialize_with = #fn_name)]
            });
            if let Some(type_param) = generate_bounds_for {
                let bound: syn::TypeParam = parse_quote! {
                    #type_param: for<'d> Deserialize<'d>
                };
                let bound = bound.into_token_stream().to_string();

                field.field.attrs.push(parse_quote! {
                    #[serde(bound(deserialize = #bound))]
                });
            }

            fn_defs.push(render_deserialize_fn(&fn_ident, &field.len));
        }
    }

    let expanded = quote! {
        #input
        #(#fn_defs)*
    };

    proc_macro::TokenStream::from(expanded)
}
