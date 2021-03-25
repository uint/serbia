use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Expr, ExprLit, Field, Ident, Lit, Meta, MetaList, NestedMeta, Type};

enum Arg {
    Flag(String),
    KeyValueArg(KeyValueArg),
}

struct KeyValueArg {
    key: String,
    value: Lit,
}

fn parse_arg(meta: NestedMeta) -> Result<Arg, ()> {
    if let NestedMeta::Meta(Meta::NameValue(meta)) = meta {
        let key = meta
            .path
            .get_ident()
            .expect("expected serbia argument to be an ident")
            .to_string();
        let value = meta.lit;

        return Ok(Arg::KeyValueArg(KeyValueArg { key, value }));
    } else if let NestedMeta::Meta(Meta::Path(path)) = meta {
        let key = path
            .get_ident()
            .expect("expected serbia flag to be an ident")
            .to_string();
        return Ok(Arg::Flag(key));
    }

    Err(())
}

/// A field that is a (potentially) big array, with convenient metadata
/// for generating custom serialization/deserialization code.
pub struct BigArrayField<'f> {
    pub field: &'f mut Field,
    pub len: TokenStream,
    pub serialize: bool,
    pub deserialize: bool,
}

impl<'f> BigArrayField<'f> {
    pub fn parse_field(field: &'f mut Field) -> Option<Self> {
        let mut len = None;
        let mut serialize = true;
        let mut deserialize = true;

        // TODO: replace with drain_filter once stabilized.
        let (serbia_attrs, other_attrs): (Vec<_>, Vec<_>) =
            field.attrs.iter().cloned().partition(|a| {
                if let Some(ident) = a.path.get_ident() {
                    return ident == "serbia";
                }

                false
            });

        field.attrs = other_attrs;

        let serde_attrs = field.attrs.iter().filter(|a| {
            if let Some(ident) = a.path.get_ident() {
                return ident == "serde";
            }

            false
        });

        for attr in serbia_attrs {
            if let Meta::List(MetaList { nested: meta, .. }) = attr.parse_meta().unwrap() {
                for arg in meta {
                    let arg = parse_arg(arg).unwrap();

                    match arg {
                        Arg::Flag(flag) => match flag.as_str() {
                            "skip" => return None,
                            "skip_serializing" => serialize = false,
                            "skip_deserializing" => deserialize = false,
                            unknown => panic!("unknown serbia flag: {}", unknown),
                        },
                        Arg::KeyValueArg(key_value) => match key_value.key.as_str() {
                            "bufsize" => len = Some(key_value.value),
                            unknown => panic!("unknown serbia key-value option: {}", unknown),
                        },
                    }
                }
            }
        }

        for attr in serde_attrs {
            if let Meta::List(MetaList { nested: meta, .. }) = attr.parse_meta().unwrap() {
                for arg in meta {
                    if let NestedMeta::Meta(meta) = arg {
                        match meta.path().get_ident().unwrap().to_string().as_str() {
                            "serialize_with" | "skip_serializing" => serialize = false,
                            "deserialize_with" | "skip_deserializing" => deserialize = false,
                            "skip" | "with" => {
                                serialize = false;
                                deserialize = false;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if let Some(len) = len {
            let len = if let Lit::Str(const_name) = len {
                Ident::new(&const_name.value(), const_name.span()).to_token_stream()
            } else {
                len.into_token_stream()
            };

            return Some(BigArrayField {
                field,
                len,
                serialize,
                deserialize,
            });
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
                    return Some(BigArrayField {
                        field,
                        len,
                        serialize,
                        deserialize,
                    });
                }
            } else if let Expr::Path(len) = &array_type.len {
                let len = len.into_token_stream();
                return Some(BigArrayField {
                    field,
                    len,
                    serialize,
                    deserialize,
                });
            }
        }

        None
    }
}
