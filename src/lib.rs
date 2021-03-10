extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, parse_quote, Attribute, Expr, ExprLit, Field, ItemEnum, ItemStruct, Lit,
    Meta, NestedMeta, Type,
};

fn render_serialize_fn(fn_ident: &Ident, ty: &Type, len: usize) -> TokenStream {
    quote! {
        fn #fn_ident<S>(array: &#ty, serializer: S) -> core::result::Result<S::Ok, S::Error>
        where
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

fn render_deserialize_fn(fn_ident: &Ident, ty: &Type, len: usize) -> TokenStream {
    let count = 0..len;

    quote! {
        fn #fn_ident<'de, D>(deserializer: D) -> core::result::Result<#ty, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct ArrayVisitor;

            impl<'de> serde::de::Visitor<'de> for ArrayVisitor {
                type Value = #ty;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str(std::concat!("an array of length ", #len))
                }

                #[inline]
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    Ok([
                        #(
                            match seq.next_element()? {
                                Some(val) => val,
                                None => return Err(serde::de::Error::invalid_length(#count, &self)),
                            }
                        ),*
                    ])
                }
            }

            deserializer.deserialize_tuple(#len, ArrayVisitor)
        }
    }
}

/// A helper that verifies the type of the field is an array larger than 32
/// and extracts its length.
fn parse_big_array(field: &mut Field) -> Option<(&mut Field, usize)> {
    // And this is how you end up in destructuring bind hell.
    if let Type::Array(array_type) = &field.ty {
        if let Expr::Lit(ExprLit {
            lit: Lit::Int(len), ..
        }) = &array_type.len
        {
            let len: usize = len.base10_parse().unwrap();

            if len > 32 {
                return Some((field, len));
            }
        }
    }

    None
}

/// Helper to search through a list of attributes for Serialize and Deserialize derives.
fn check_if_serializing_deserializing<'a>(
    attrs: impl Iterator<Item = &'a Attribute>,
) -> (bool, bool) {
    let mut serialize = false;
    let mut deserialize = false;

    for derive_attr in attrs.filter(|attr| attr.path.is_ident("derive")) {
        if let Meta::List(derive_attr) = derive_attr.parse_meta().unwrap() {
            for derive in derive_attr.nested {
                if let NestedMeta::Meta(Meta::Path(path)) = derive {
                    // TODO: Is there a better way to make sure these are the derives we want?
                    if let Some(last_segment) = path.segments.iter().last() {
                        if last_segment.ident == "Serialize" {
                            serialize = true;
                        } else if last_segment.ident == "Deserialize" {
                            deserialize = true;
                        }
                    }
                };
            }
        }
    }

    (serialize, deserialize)
}

enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Item {
    fn fields(&mut self) -> impl Iterator<Item = &mut Field> {
        let result: Box<dyn Iterator<Item = &mut Field>> = match self {
            Item::Struct(s) => Box::new(s.fields.iter_mut()),
            Item::Enum(e) => {
                let outer_iter = e.variants.iter_mut();
                let result = outer_iter.map(|v| v.fields.iter_mut()).flatten();
                Box::new(result)
            }
        };

        result
    }

    fn attrs(&self) -> impl Iterator<Item = &Attribute> {
        match self {
            Item::Struct(s) => s.attrs.iter(),
            Item::Enum(e) => e.attrs.iter(),
        }
    }

    fn ident(&self) -> &Ident {
        match self {
            Item::Struct(s) => &s.ident,
            Item::Enum(e) => &e.ident,
        }
    }
}

impl Parse for Item {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: syn::Item = input.parse()?;

        match item {
            syn::Item::Struct(s) => Ok(Self::Struct(s)),
            syn::Item::Enum(e) => Ok(Self::Enum(e)),
            _ => Err(syn::Error::new(
                input.span(),
                "serbia accepts only enums or structs",
            )),
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Item::Struct(s) => s.to_tokens(tokens),
            Item::Enum(e) => e.to_tokens(tokens),
        }
    }
}

/// An attribute macro that enables big arrays for [Serde](serde).
///
/// Simply slap it on top of your struct or enum, before the [Serialize](serde::Serialize)/[Deserialize](serde::Deserialize) derive.
///
/// # Usage
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
#[proc_macro_attribute]
pub fn serbia(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Item);

    let struct_name = input.ident().to_string();

    // Determine whether we need to generate code for serialization and/or deserialization.
    let external_attrs = input.attrs();
    let (serialize, deserialize) = check_if_serializing_deserializing(external_attrs);

    let mut fn_defs = vec![];

    for (i, (field, len)) in input.fields().filter_map(parse_big_array).enumerate() {
        let ty = &field.ty;

        if serialize {
            let fn_ident = format_ident!("serbia_serialize_{}_arr_{}", struct_name, i);
            let fn_name = fn_ident.to_string();

            field.attrs.push(parse_quote! {
                #[serde(serialize_with = #fn_name)]
            });

            fn_defs.push(render_serialize_fn(&fn_ident, &ty, len));
        }
        if deserialize {
            let fn_ident = format_ident!("serbia_deserialize_{}_arr_{}", struct_name, i);
            let fn_name = fn_ident.to_string();

            field.attrs.push(parse_quote! {
                #[serde(deserialize_with = #fn_name)]
            });

            fn_defs.push(render_deserialize_fn(&fn_ident, &ty, len));
        }
    }

    let expanded = quote! {
        #input
        #(#fn_defs)*
    };

    proc_macro::TokenStream::from(expanded)
}

#[test]
fn test_parse_big_array() {
    let s: ItemStruct = parse_quote! {
        struct S {
            a: String,
            b: [u32; 32],
            c: [u32; 33],
        }
    };

    let mut fields: Vec<_> = s.fields.into_iter().collect();

    assert!(parse_big_array(&mut fields[0]).is_none());
    assert!(parse_big_array(&mut fields[1]).is_none());
    assert!(parse_big_array(&mut fields[2]).unwrap().1 == 33);
}

#[test]
fn test_no_serde_derive() {
    let attrs: Vec<Attribute> = vec![
        parse_quote! {
            #[derive(Serializer, Debug, Asd)]
        },
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[serde(serialize_with = "asd")]
        },
    ];

    assert_eq!(
        check_if_serializing_deserializing(attrs.iter()),
        (false, false)
    );
}

#[test]
fn test_detect_serialize() {
    let attrs: Vec<Attribute> = vec![
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[derive(Deserializer, Debug, Asd, Serialize)]
        },
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[serde(serialize_with = "asd")]
        },
    ];

    assert_eq!(
        check_if_serializing_deserializing(attrs.iter()),
        (true, false)
    );
}

#[test]
fn test_detect_deserialize() {
    let attrs: Vec<Attribute> = vec![
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[derive(Deserializer, Debug, Deserialize, Asd)]
        },
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[serde(serialize_with = "asd")]
        },
    ];

    assert_eq!(
        check_if_serializing_deserializing(attrs.iter()),
        (false, true)
    );
}

#[test]
fn test_detect_serialize_deserialize() {
    let attrs: Vec<Attribute> = vec![
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[derive(Serialize, Deserializer, Debug, Deserialize, Asd)]
        },
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[serde(serialize_with = "asd")]
        },
    ];

    assert_eq!(
        check_if_serializing_deserializing(attrs.iter()),
        (true, true)
    );
}

#[test]
fn test_detect_serialize_deserialize_qualified() {
    let attrs: Vec<Attribute> = vec![
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[derive(serde::Serialize, Deserializer, Debug, serde::Deserialize, Asd)]
        },
        parse_quote! {
            #[asd]
        },
        parse_quote! {
            #[serde(serialize_with = "asd")]
        },
    ];

    assert_eq!(
        check_if_serializing_deserializing(attrs.iter()),
        (true, true)
    );
}
