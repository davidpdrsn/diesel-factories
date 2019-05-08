#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use proc_macro2::Span;
use quote::quote;
use regex::Regex;
use syn::punctuated::Pair;
use syn::Ident;
use syn::{parse_macro_input, Attribute, DeriveInput};
use syn::{Data, Fields, FieldsNamed};

/// See the docs for "diesel_factories" for more info about this.
#[proc_macro_derive(Factory, attributes(factory_model))]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let model_name = model_name(&input.attrs);
    let factory_name = input.ident.clone();
    let fields = struct_fields(input)
        .named
        .into_pairs()
        .map(|pair| match pair {
            Pair::Punctuated(field, _) => field,
            Pair::End(field) => field,
        });

    let methods = fields
        .clone()
        .map(|field| {
            let name = field.ident.unwrap_or_else(|| panic!("Field without name"));
            let ty = field.ty;
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
        .clone()
        .filter_map(|field| {
            let name = field.ident.unwrap_or_else(|| panic!("Field without name"));
            let ty = field.ty;
            if (name != "connection") {
                Some(quote! {
                    (#name.eq(&self.#name))
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let tokens = quote! {
    impl<'a> #factory_name<'a> {

        #(#methods)*


        fn insert(self) -> #model_name {
            use self::users::dsl::*;
            let res = diesel::insert_into(users)
                .values( #(#diesel_tuples)* )
                .get_result::<#model_name>(self.connection);

            match res {
                Ok(x) => x,
                Err(err) => panic!("{}", err),
            }
        }
    }
    };
    println!("{}", tokens);
    tokens.into()
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
