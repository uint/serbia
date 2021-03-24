use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, ExprLit, Field, Lit, Meta, MetaList, Type};

/// A field that is a (potentially) big array, with convenient metadata
/// for generating custom serialization/deserialization code.
pub struct BigArrayField<'f> {
    pub field: &'f mut Field,
    pub len: TokenStream,
}

impl<'f> BigArrayField<'f> {
    pub fn from_field(field: &'f mut Field) -> Option<Self> {
        if let Some(i) = field.attrs.iter().position(|a| {
            if let Some(ident) = a.path.get_ident() {
                return ident == "serbia_bufsize";
            }

            false
        }) {
            let attr = field.attrs.remove(i);

            if let Meta::List(MetaList {
                nested: mut meta, ..
            }) = attr.parse_meta().unwrap()
            {
                if meta.len() != 1 {
                    panic!("serbia_bufsize expected 1 argument, found {}", meta.len());
                }

                let len = meta.pop().unwrap().into_token_stream();

                return Some(Self { field, len });
            }
        }

        // And this is how you end up in destructuring bind hell.
        if let Type::Array(array_type) = &field.ty {
            if let Expr::Lit(ExprLit {
                lit: Lit::Int(len), ..
            }) = &array_type.len
            {
                let len: usize = len.base10_parse().unwrap();

                if len > 32 {
                    let len = array_type.len.clone().into_token_stream();
                    return Some(BigArrayField { field, len });
                }
            } else if let Expr::Path(len) = &array_type.len {
                let len = len.into_token_stream();
                return Some(BigArrayField { field, len });
            }
        }

        None
    }
}
