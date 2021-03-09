extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, ExprLit, Field, ItemStruct, Lit, Meta, NestedMeta, Type, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn serbia(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    let struct_name = input.ident.to_string();

    // Determine whether we need to generate code for serialization and/or deserialization.
    let mut serialize = false;
    let mut deserialize = false;

    for derive_attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("derive"))
    {
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

    let mut serialize_fn_defs = vec![];
    // let mut deserialize_fn_defs = vec![];

    for (i, (field, len)) in input.fields.iter_mut().filter_map(parse_big_array).enumerate() {
        let ty = &field.ty;

        if serialize {
            let fn_ident = format_ident!("serbia_serialize_{}_arr_{}", struct_name, i);
            let fn_name = fn_ident.to_string();

            field.attrs.push(parse_quote! {
                #[serde(serialize_with = #fn_name)]
            });

            serialize_fn_defs.push(quote! {
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
            });
        }
        if deserialize {
            let fn_name = format_ident!("serbia_deserialize_{}_arr_{}", struct_name, i);

            field.attrs.push(parse_quote! {
                #[serde(deserialize_with = #fn_name)]
            });
        }
    }

    let expanded = quote! {
        #input
        #(#serialize_fn_defs)*
    };

    TokenStream::from(expanded)
}
