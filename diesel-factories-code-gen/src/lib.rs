//! See the docs for "diesel-factories" for more info about this.

#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;

use darling::FromDeriveInput;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::PathSegment;
use syn::Type::Path;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Factory, attributes(factory))]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let options = match Options::from_derive_input(&ast) {
        Ok(options) => options,
        Err(err) => panic!("{}", err),
    };

    let out = DeriveData::new(ast, options);
    let tokens = out.build_derive_output();
    tokens.into()
}

#[derive(FromDeriveInput, Debug)]
#[darling(attributes(factory), forward_attrs(doc, cfg, allow))]
struct Options {
    model: syn::Ident,
    #[darling(default)]
    connection: Option<syn::Path>,
    #[darling(default)]
    id: Option<syn::Ident>,
    table: syn::Path,
}

struct DeriveData {
    input: DeriveInput,
    options: Options,
    tokens: TokenStream,
}

impl DeriveData {
    fn new(input: DeriveInput, options: Options) -> Self {
        Self {
            input,
            options,
            tokens: quote! {},
        }
    }

    fn build_derive_output(mut self) -> TokenStream {
        self.gen_factory_methods_impl();
        self.gen_builder_methods();
        self.gen_set_association_traits();

        self.tokens
    }

    fn gen_factory_methods_impl(&mut self) {
        let factory = self.factory_name();
        let generics = self.factory_generics();
        let model_type = self.model_type();
        let id_type = self.id_type();
        let connection_type = self.connection_type();
        let table_path = self.table_path();
        let values = self.diesel_insert_values();

        self.tokens.extend(quote! {
            impl#generics diesel_factories::Factory for #factory#generics {
                type Model = #model_type;
                type Id = #id_type;
                type Connection = #connection_type;

                fn insert(self, con: &Self::Connection) -> Self::Model {
                    use #table_path::dsl::*;
                    use #table_path as table;

                    use diesel::prelude::*;
                    let values = ( #(#values),* );
                    diesel::insert_into(table::table)
                        .values(values)
                        .get_result::<Self::Model>(con)
                        .unwrap()
                }

                fn id_for_model(model: &Self::Model) -> &Self::Id {
                    &model.id
                }
            }
        });
    }

    fn gen_builder_methods(&mut self) {
        let factory = self.factory_name();
        let generics = self.factory_generics();
        let methods = self.builder_methods();

        self.tokens.extend(quote! {
            impl#generics #factory#generics {
                #(#methods)*
            }
        })
    }

    fn factory_name(&self) -> &syn::Ident {
        &self.input.ident
    }

    fn model_type(&self) -> &syn::Ident {
        &self.options.model
    }

    fn id_type(&self) -> TokenStream {
        self.options
            .id
            .as_ref()
            .map(|inner| quote! { #inner })
            .unwrap_or_else(|| quote! { i32 })
    }

    fn connection_type(&self) -> TokenStream {
        self.options
            .connection
            .as_ref()
            .map(|inner| quote! { #inner })
            .unwrap_or_else(|| quote! { diesel::pg::PgConnection })
    }

    fn table_path(&self) -> &syn::Path {
        &self.options.table
    }

    fn factory_generics(&self) -> &syn::Generics {
        &self.input.generics
    }

    fn struct_fields(&self) -> syn::punctuated::Iter<syn::Field> {
        use syn::{Data, Fields};

        match &self.input.data {
            Data::Union(_) => panic!("Factory can only be derived on structs"),
            Data::Enum(_) => panic!("Factory can only be derived on structs"),
            Data::Struct(data) => match &data.fields {
                Fields::Named(named) => named.named.iter(),
                Fields::Unit => panic!("Factory can only be derived on structs with named fields"),
                Fields::Unnamed(_) => {
                    panic!("Factory can only be derived on structs with named fields")
                }
            },
        }
    }

    fn diesel_insert_values(&self) -> Vec<TokenStream> {
        self.struct_fields()
            .map(|field| self.diesel_insert_value(field))
            .collect()
    }

    fn diesel_insert_value(&self, field: &syn::Field) -> TokenStream {
        let name = field
            .ident
            .as_ref()
            .unwrap_or_else(|| panic!("Factory can only be derived for named fields"));

        if let Some(association) = self.parse_association_type(&field.ty) {
            let foreign_key_field = ident(&format!("{}_id", name));
            if association.is_option {
                quote! {
                    {
                        let value = self.#name.map(|inner| {
                            inner.insert_returning_id(con)
                        });
                        #foreign_key_field.eq(value)
                    }
                }
            } else {
                quote! {
                    #foreign_key_field.eq(self.#name.insert_returning_id(con))
                }
            }
        } else {
            quote! {
                #name.eq(&self.#name)
            }
        }
    }

    fn is_association_field(&self, ty: &syn::Type) -> bool {
        let as_string = self.type_to_string(ty);
        as_string.contains("Association <")
    }

    fn type_to_string(&self, ty: &syn::Type) -> String {
        use quote::ToTokens;

        let mut tokenized = quote! {};
        ty.to_tokens(&mut tokenized);
        tokenized.to_string()
    }

    fn builder_methods(&self) -> Vec<TokenStream> {
        self.struct_fields()
            .filter_map(|field| self.builder_method(field))
            .collect()
    }

    fn builder_method(&self, field: &syn::Field) -> Option<TokenStream> {
        let name = &field.ident;
        let ty = &field.ty;

        if self.is_association_field(&field.ty) {
            None
        } else {
            Some(quote! {
                #[allow(missing_docs, dead_code)]
                pub fn #name<T: Into<#ty>>(mut self, t: T) -> Self {
                    self.#name = t.into();
                    self
                }
            })
        }
    }

    fn gen_set_association_traits(&mut self) {
        let association_traits = self.association_traits();

        self.tokens.extend(quote! {
            #(#association_traits)*
        });
    }

    fn association_traits(&self) -> Vec<TokenStream> {
        self.struct_fields()
            .filter_map(|field| self.association_trait(field))
            .collect()
    }

    fn association_trait(&self, field: &syn::Field) -> Option<TokenStream> {
        use heck::CamelCase;

        if self.is_association_field(&field.ty) {
            let factory = self.factory_name();
            let field_name = field.ident.as_ref().expect("field without name");
            let camel_field_name = field_name.to_string().to_camel_case();

            let association = self.parse_association_type(&field.ty).unwrap_or_else(|| {
                use std::fmt::Write;
                let mut s = String::new();
                writeln!(
                    s,
                    "Invalid association attribute. Must be on one of the following forms"
                )
                .unwrap();
                writeln!(s).unwrap();
                writeln!(s, "Association<'a, Model, Factory>").unwrap();
                writeln!(s, "Option<Association<'a, Model, Factory>>").unwrap();
                writeln!(s, "Association<'a, Model, Factory<'a>>").unwrap();
                writeln!(s, "Option<Association<'a, Model, Factory<'a>>>").unwrap();
                writeln!(s).unwrap();
                writeln!(s, "Got\n{}", self.type_to_string(&field.ty)).unwrap();
                panic!("{}", s);
            });

            let model = association.model;
            let other_factory = association.factory;

            let other_factory_without_lifetime =
                self.type_to_string(&other_factory).replace(" < 'a >", "");
            let trait_name = ident(&format!(
                "Set{}On{}For{}",
                other_factory_without_lifetime, factory, camel_field_name
            ));

            let model_impl = if association.is_option {
                quote! {
                    impl<'a> #trait_name<Option<&'a #model>> for #factory<'a> {
                        fn #field_name(mut self, t: Option<&'a #model>) -> Self {
                            self.#field_name = t.map(|k| diesel_factories::Association::new_model(k));
                            self
                        }
                    }
                }
            } else {
                quote! {
                    impl<'a> #trait_name<&'a #model> for #factory<'a> {
                        fn #field_name(mut self, t: &'a #model) -> Self {
                            self.#field_name = diesel_factories::Association::new_model(t);
                            self
                        }
                    }
                }
            };

            let factory_impl = if association.is_option {
                quote! {
                    impl<'a> #trait_name<Option<#other_factory>> for #factory<'a> {
                        fn #field_name(mut self, t: Option<#other_factory>) -> Self {
                            self.#field_name = t.map(|k| diesel_factories::Association::new_factory(k));
                            self
                        }
                    }
                }
            } else {
                quote! {
                    impl<'a> #trait_name<#other_factory> for #factory<'a> {
                        fn #field_name(mut self, t: #other_factory) -> Self {
                            self.#field_name = diesel_factories::Association::new_factory(t);
                            self
                        }
                    }
                }
            };

            Some(quote! {
                #[allow(missing_docs, dead_code)]
                pub trait #trait_name<T> {
                    fn #field_name(self, t: T) -> Self;
                }

                #model_impl

                #factory_impl
            })
        } else {
            None
        }
    }

    fn option_detected(&self, ty: &syn::Type) -> bool {
        if let Path(syn::TypePath { qself: _, path }) = ty.clone() {
            let syn::Path {
                leading_colon: _,
                segments,
            } = path;

            let path_segment = segments.last().unwrap().value().clone();
            if path_segment.ident.to_string() == "Option" {
                return true;
            // println!("Optional detected {}", self.type_to_string(ty))
            } else {
                return false;
                // println!("Optional NOT detected {}", self.type_to_string(ty))
            }
        } else {
            panic!("Expected a TypePath here");
        }
    }

    fn parse_association_type(&self, ty: &syn::Type) -> Option<Association> {
        println!("START PARSE");
        let is_option = self.option_detected(ty);

        use regex::Regex;

        let re = Regex::new(
            r"(Option < )?Association < 'a , (?P<model>[^ ]+) , (?P<factory>[^ ]+( < 'a >)?) >( >)?",
        )
        .unwrap();
        let as_string = self.type_to_string(ty);
        let caps = re.captures(&as_string)?;

        let model = &caps["model"];
        let model = syn::parse_str::<syn::Type>(model).unwrap_or_else(|e| {
            panic!("{}", e);
        });

        let factory = &caps["factory"];
        let factory = syn::parse_str::<syn::Type>(factory).unwrap_or_else(|e| {
            panic!("{}", e);
        });

        Some(Association {
            is_option,
            model,
            factory,
        })
    }
}

fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, Span::call_site())
}

struct Association {
    is_option: bool,
    model: syn::Type,
    factory: syn::Type,
}
