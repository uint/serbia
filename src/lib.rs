extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Attribute, Expr, ExprLit, Field, ItemStruct, Lit, Meta,
    NestedMeta, Type,
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
                    // TODO: What happens if we get something like #[derive(serde::Serialize)]?
                    if path.is_ident("Serialize") {
                        serialize = true;
                    } else if path.is_ident("Deserialize") {
                        deserialize = true;
                    }
                };
            }
        }
    }

    (serialize, deserialize)
}

#[proc_macro_attribute]
pub fn serbia(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);

    let struct_name = input.ident.to_string();

    // Determine whether we need to generate code for serialization and/or deserialization.
    let external_attrs = input.attrs.iter();
    let (serialize, deserialize) = check_if_serializing_deserializing(external_attrs);

    let mut fn_defs = vec![];

    for (i, (field, len)) in input
        .fields
        .iter_mut()
        .filter_map(parse_big_array)
        .enumerate()
    {
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
