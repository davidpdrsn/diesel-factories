#![recursion_limit = "128"]

extern crate diesel;
extern crate proc_macro;
extern crate proc_macro2;

extern crate heck;

use heck::SnakeCase;
use proc_macro2::Span;
use quote::quote;
use quote::ToTokens;
use regex::Regex;
use syn::punctuated::Pair;
use syn::Ident;
use syn::{parse_macro_input, Attribute, DeriveInput};
use syn::{Data, Fields, FieldsNamed};

/// See the docs for "diesel_factories" for more info about this.
#[proc_macro_derive(Factory, attributes(factory_model, table_name, factory))]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let model_name = model_name(&input.attrs);
    let table_name = table_name(&input.attrs);
    let factory_name = input.ident.clone();
    let fields = struct_fields(input)
        .named
        .into_pairs()
        .map(|pair| match pair {
            Pair::Punctuated(field, _) => field,
            Pair::End(field) => field,
        })
        .collect::<Vec<_>>();

    let methods = fields
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .unwrap_or_else(|| panic!("Field without name"));
            let ty = &field.ty;
            quote! {
                #[allow(missing_docs)]
                pub fn #name<T: Into<#ty>>(mut self, value: T) -> Self {
                    self.#name = value.into();
                    self
                }
            }
        })
        .collect::<Vec<_>>();

    let diesel_tuples = fields
        .iter()
        .filter_map(|field| {
            let name = field
                .ident
                .as_ref()
                .unwrap_or_else(|| panic!("Field without name"));

            if name != "connection" {
                Some(quote! {
                    (#name.eq(&self.#name))
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let association_field_impls = fields
        .iter()
        .filter_map(|field| {
            let name = field.ident.as_ref().unwrap_or_else(|| panic!("Field without name"));
            let ty = &field.ty;
            let type_string = ty.into_token_stream().to_string();
            let re = Regex::new(r"^Option < .* >$").unwrap();
            let options = re.captures(&type_string);
            let optional = options.is_some();
            let maybe_factory_attr = field.attrs.iter().find(|attr| {
                attr.path
                    .segments
                    .iter()
                    .any(|segment| &segment.ident.to_string() == "factory")
            });

            if let Some(factory_attr) = maybe_factory_attr {
                let attr = factory_attr.tts.to_string();
                // FIXME, unicode is a valid identifier in rust!
                let re = Regex::new(r"model = ([A-Za-z]+) ").unwrap();
                let model_cap = re.captures(&attr).unwrap();
                let model = &model_cap[1];
                let model_fn = ident(&model.to_snake_case());
                let model = ident(model);

                if optional {
                    return Some(quote! {
                        fn #model_fn<T: diesel_factories::Association<#model_name, #model>>(mut self, association: &T) -> Self {
                            self.#name = Some(association.id());
                            self
                        }
                    });
                } else {
                    return Some(quote! {
                        fn #model_fn<T: diesel_factories::Association<#model_name, #model>>(mut self, association: &T) -> Self {
                            self.#name = association.id();
                            self
                        }
                    });
                }
            }

            return None;
        })
        .collect::<Vec<_>>();

    let association_impls = fields
        .iter()
        .filter_map(|field| {
            let maybe_factory_attr = field.attrs.iter().find(|attr| {
                attr.path
                    .segments
                    .iter()
                    .any(|segment| &segment.ident.to_string() == "factory")
            });

            if let Some(factory_attr) = maybe_factory_attr {
                let attr = factory_attr.tts.to_string();
                // FIXME, unicode is a valid identifier in rust!
                let re = Regex::new(r"model = ([A-Za-z]+) ").unwrap();
                let model_cap = re.captures(&attr).unwrap();
                let re = Regex::new(r"factory = ([A-Za-z]+) ").unwrap();
                let factory_cap = re.captures(&attr).unwrap();
                let model = ident(&model_cap[1]);
                let factory = ident(&factory_cap[1]);
                return Some(quote! {
                    impl<'a> diesel_factories::Association<#model_name, #model> for #factory<'a> {
                        fn id(&self) -> i32 {
                            self.insert().id
                        }
                    }

                    impl diesel_factories::Association<#model_name, #model> for #model {
                        fn id(&self) -> i32 {
                            self.id
                        }
                    }
                });
            }

            return None;
        })
        .collect::<Vec<_>>();
    let combined_diesel_tuples = quote! { #(#diesel_tuples),* };
    let tokens = quote! {
        impl<'a> #factory_name<'a> {
            #(#methods)*

            #(#association_field_impls)*

            fn insert(&self) -> #model_name {
                use crate::schema::#table_name::dsl::*;
                diesel::insert_into(#table_name)
                    .values(( #(#combined_diesel_tuples)* ))
                    .get_result::<#model_name>(self.connection).unwrap()
            }
        }

        #(#association_impls)*
    };
    tokens.into()
}

fn table_name(attrs: &Vec<Attribute>) -> Ident {
    let table_model_attr = attrs.into_iter().find(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| &segment.ident.to_string() == "table_name")
    });

    let table_model_attr = match table_model_attr {
        Some(x) => x,
        None => {
            panic!("#[derive(Factory)] requires you to also set the attribute #[table_name(...)]")
        }
    };

    let attr = table_model_attr.tts.to_string();

    let re = Regex::new(r"[a-z]+").unwrap();
    let caps = re
        .captures(&attr)
        .expect("The `table_name` attributes must be on the form `#[table_name = \"...\"]`");

    ident(&caps[0].to_string())
}

fn model_name(attrs: &Vec<Attribute>) -> Ident {
    let factory_model_attr = attrs.into_iter().find(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| &segment.ident.to_string() == "factory_model")
    });

    let factory_model_attr = match factory_model_attr {
        Some(x) => x,
        None => panic!(
            "#[derive(Factory)] requires you to also set the attribute #[factory_model(...)]"
        ),
    };

    let attr = factory_model_attr.tts.to_string();
    let re = Regex::new(r"\( (?P<name>.*?) \)").unwrap();
    let caps = re.captures(&attr).expect(
        "The `factory_model` attributes must be on the form `#[factory_model(SomeStruct)]`",
    );

    ident(&caps["name"])
}

fn ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

fn struct_fields(input: DeriveInput) -> FieldsNamed {
    let err = || panic!("Factory can only be derived on structs with named fields");

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields,
            _ => err(),
        },
        _ => err(),
    }
}
