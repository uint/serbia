extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{Expr, ExprLit, Field, ItemStruct, Lit, Meta, NestedMeta, Type, parse_macro_input};
use quote::quote;

#[proc_macro_attribute]
pub fn serbia(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    // Determine whether we need to generate code for serialization and/or deserialization.
    let mut serialize = false;
    let mut deserialize = false;

    for derive_attr in input.attrs.iter().filter(|attr| attr.path.is_ident("derive")) {
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
            if let Expr::Lit(ExprLit {lit: Lit::Int(len), ..}) = &array_type.len {
                let len: usize = len.base10_parse().unwrap();

                if len > 32 {
                    return true;
                }
            }
        }

        false
    } 

    for (i, field) in input.fields.iter_mut().filter(is_big_array).enumerate() {
        
    }

    let expanded = quote! {
        #input
    };

    TokenStream::from(expanded)
}
