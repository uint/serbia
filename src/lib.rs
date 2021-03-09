extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Expr, ExprLit, Field, ItemStruct, Lit, Meta, NestedMeta, Type,
};

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

    fn is_big_array(field: &&mut Field) -> bool {
        // And this is how you end up in destructuring bind hell.
        if let Type::Array(array_type) = &field.ty {
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(len), ..
            }) = &array_type.len
            {
                let len: usize = len.base10_parse().unwrap();

                if len > 32 {
                    return true;
                }
            }
        }

        false
    }

    // let serialize_fn_defs = vec![];
    // let deserialize_fn_defs = vec![];

    for (i, field) in input.fields.iter_mut().filter(is_big_array).enumerate() {
        if serialize {
            let fn_name = format!("serbia_serialize_{}_arr_{}", struct_name, i);

            field.attrs.push(parse_quote! {
                #[serde(serialize_with = #fn_name)]
            });
        }
        if deserialize {
            let fn_name = format!("serbia_deserialize_{}_arr_{}", struct_name, i);

            field.attrs.push(parse_quote! {
                #[serde(deserialize_with = #fn_name)]
            });
        }
    }

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}
