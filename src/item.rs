mod fields;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};
use syn::{Attribute, Field, ItemEnum, ItemStruct, Meta, NestedMeta};

use fields::BigArrayField;

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

pub struct Context {
    pub type_name: String,
    pub serialize: bool,
    pub deserialize: bool,
}

pub enum Item {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Item {
    pub fn ident(&self) -> &Ident {
        match self {
            Item::Struct(s) => &s.ident,
            Item::Enum(e) => &e.ident,
        }
    }

    pub fn context(&self) -> Context {
        let (serialize, deserialize) = check_if_serializing_deserializing(self.attrs());

        Context {
            type_name: self.ident().to_string(),
            serialize,
            deserialize,
        }
    }

    pub fn big_array_fields(&mut self) -> impl Iterator<Item = BigArrayField> {
        self.fields().filter_map(BigArrayField::from_field)
    }

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

#[cfg(test)]
mod tests {
    use syn::{parse_quote, Attribute, ItemStruct};

    use super::{check_if_serializing_deserializing, BigArrayField};

    #[test]
    fn parse_big_array_len() {
        let s: ItemStruct = parse_quote! {
            struct S {
                a: String,
                b: [u32; 32],
                c: [u32; 33],
            }
        };

        let mut fields: Vec<_> = s.fields.into_iter().collect();

        assert!(BigArrayField::from_field(&mut fields[0]).is_none());
        assert!(BigArrayField::from_field(&mut fields[1]).is_none());
        assert!(BigArrayField::from_field(&mut fields[2]).is_some());
    }

    #[test]
    fn manual_bufsize() {
        let s: ItemStruct = parse_quote! {
            struct S {
                a: String,
                #[serbia_bufsize(32)]
                b: [u32; 32],
                c: [u32; 33],
            }
        };

        let mut fields: Vec<_> = s.fields.into_iter().collect();

        assert!(BigArrayField::from_field(&mut fields[0]).is_none());
        assert!(BigArrayField::from_field(&mut fields[1]).is_some());
        assert!(BigArrayField::from_field(&mut fields[2]).is_some());
    }

    #[test]
    fn no_serde_derive() {
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
    fn detect_serialize() {
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
    fn detect_deserialize() {
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
    fn detect_serialize_deserialize() {
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
    fn detect_serialize_deserialize_qualified() {
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
}
