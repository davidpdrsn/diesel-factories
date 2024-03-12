//! See the docs for "diesel-factories" for more info about this.

#![deny(mutable_borrow_reservation_conflict)]
#![recursion_limit = "128"]
#![deny(
    mutable_borrow_reservation_conflict,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use heck::CamelCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::{format_ident, ToTokens};
use syn::spanned::Spanned;


use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    GenericArgument, Ident, ItemStruct, Lifetime, Path, PathArguments, PathSegment, Token, Type,
};

#[proc_macro_derive(Factory, attributes(factory))]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    let tokens = quote! { #input };
    proc_macro::TokenStream::from(tokens)
}

struct MapInput {
    ident: Ident,
    model: Type,
    fields: Vec<Ident>,
}

impl Parse for MapInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ItemStruct {
            attrs,
            ident,
            generics: _,
            fields: item_strut_fields,
            struct_token: _,
            semi_token: _,
            vis: _,
        } = input.parse::<ItemStruct>()?;

        let mut fields: Vec<Ident> = Vec::new();

        let struct_attr::MapModel {
            model,
        } = struct_attr::MapModel::from_attributes(&attrs)?;

        for field in item_strut_fields {
            let field_span = field.span();

            let name = field
                .ident
                .ok_or_else(|| syn::Error::new(field_span, "Unnamed fields are not supported"))?;

            fields.push(name);
        }

        Ok(MapInput {
            fields,
            ident,
            model,
        })
    }
}

impl ToTokens for MapInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.factory_trait_impl());
    }
}

impl MapInput {
    fn factory_trait_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let fields = &self.fields;
        let model = &self.model;

        quote! {
            impl #ident {
                fn get_model_fields() -> (#(#model::#fields),*) {
                    (#(#model::#fields),*)
                }
            };
        }
    }
}

#[proc_macro_derive(MapModel, attributes(map_model))]
pub fn derive_map_model(input: proc_macro::TokenStream) ->  proc_macro::TokenStream { 
    let input = parse_macro_input!(input as MapInput);
    let tokens = quote! { #input };
    proc_macro::TokenStream::from(tokens)
}


mod struct_attr {
    use bae::FromAttributes;
    use syn::{Ident, Path, Type};

    #[derive(Debug, FromAttributes)]
    pub struct Factory {
        pub model: Type,
        pub table: Path,
        pub map_fields: Option<()>,
        pub connection: Option<Type>,
        pub id: Option<Type>,
        pub id_name: Option<Ident>,
    }

    #[derive(Debug, FromAttributes)]
    pub struct MapModel {
        pub model: Type,
    }
}

mod field_attr {
    use bae::FromAttributes;
    use syn::Ident;

    #[derive(Debug, FromAttributes)]
    pub struct Factory {
        pub foreign_key_name: Ident,
    }
}

#[derive(Debug)]
struct Input {
    model: Type,
    table: Path,
    connection: Type,
    id_type: Type,
    id_name: Ident,
    factory_name: Ident,
    map_fields: Option<()>,
    fields: Vec<(Ident, Type)>,
    associations: Vec<(Ident, AssociationType, Ident)>,
    lifetime: Option<Lifetime>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ItemStruct {
            attrs,
            ident: factory_name,
            generics,
            fields: item_strut_fields,
            struct_token: _,
            semi_token: _,
            vis: _,
        } = input.parse::<ItemStruct>()?;

        let struct_attr::Factory {
            model,
            table,
            connection,
            id,
            id_name,
            map_fields,
        } = struct_attr::Factory::from_attributes(&attrs)?;

        let connection =
            connection.unwrap_or_else(|| syn::parse2(quote! { diesel::pg::PgConnection }).unwrap());
        let id_type = id.unwrap_or_else(|| syn::parse2(quote! { i32 }).unwrap());
        let id_name = id_name.unwrap_or_else(|| syn::parse2(quote! { id }).unwrap());

        // parse fields and associations
        let mut fields = Vec::new();
        let mut associations = Vec::new();
        for field in item_strut_fields {
            let field_span = field.span();

            let name = field
                .ident
                .ok_or_else(|| syn::Error::new(field_span, "Unnamed fields are not supported"))?;

            let field_ty = field.ty.clone();

            if let Ok(association_type) = AssociationType::new(field_ty) {
                let foreign_key_name =
                    if let Some(attr) = field_attr::Factory::try_from_attributes(&field.attrs)? {
                        attr.foreign_key_name
                    } else {
                        format_ident!("{}_{}", name, id_name)
                    };

                associations.push((name, association_type, foreign_key_name));
            } else {
                if field_attr::Factory::from_attributes(&field.attrs).is_ok() {
                    return Err(syn::Error::new(
                        field_span,
                        "`#[factory]` attributes are only allowed on association fields",
                    ));
                }

                fields.push((name, field.ty));
            }
        }

        // parse generic lifetime
        let generics_span = generics.span();
        let mut generics_iter = generics.params.into_iter();
        let lifetime = match generics_iter.next() {
            Some(inner) => match inner {
                syn::GenericParam::Lifetime(lt_def) => {
                    if !lt_def.bounds.is_empty() {
                        return Err(syn::Error::new(lt_def.span(), "Unexpected lifetime bounds"));
                    }

                    Some(lt_def.lifetime)
                }
                _ => {
                    return Err(syn::Error::new(
                        generics_span,
                        "Expected a single generic lifetime argument",
                    ));
                }
            },
            None => None,
        };

        if let Some(arg) = generics_iter.next() {
            return Err(syn::Error::new(arg.span(), "Unexpected generic argument"));
        }

        Ok(Input {
            model,
            table,
            connection,
            id_type,
            id_name,
            factory_name,
            fields,
            associations,
            lifetime,
            map_fields
        })
    }
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.factory_trait_impl());
        tokens.extend(self.field_builder_methods());
        tokens.extend(self.association_builder_methods());
    }
}

impl Input {
    fn factory_trait_impl(&self) -> TokenStream {
        let factory = &self.factory_name;
        let lifetime = &self.lifetime;
        let model_type = &self.model;
        let id_type = &self.id_type;
        let connection_type = &self.connection;
        let table_path = &self.table;
        let id_name = &self.id_name;
        
        let insert_code = if self.no_fields() {

            match self.map_fields  {
                Some(_) => {
                    let fields = self.fields.iter().map(|(name, _)| name);
                    
                    quote! {                    
                        diesel::insert_into(#table_path::table)
                        .default_values()
                        .returning((#(#table_path::#fields),*))
                        .get_result::<Self::Model>(con)
                        .expect("Insert of factory failed")
                    }
                },
                _ => quote! {
                    diesel::insert_into(#table_path::table)
                    .default_values()
                    .get_result::<Self::Model>(con)
                    .expect("Insert of factory failed")
                }
            }
        } else {
            let values = self.fields.iter().map(|(name, _)| {
                quote! { #table_path::#name.eq(&self.#name) }
            });
            let values = values.chain(self.associations.iter().map(
                |(name, association_type, foreign_key_field)| {
                    if association_type.is_optional {
                        quote! {
                            {
                                let value = self.#name.map(|inner| {
                                    inner.insert_returning_id(con)
                                });
                                #table_path::#foreign_key_field.eq(value)
                            }
                        }
                    } else {
                        quote! {
                            #table_path::#foreign_key_field.eq(self.#name.insert_returning_id(con))
                        }
                    }
                },
            ));
            
            match self.map_fields  {
                Some(_) => {
                    let fields = self.fields.iter().map(|(name, _)| name);

                    quote! {
                        let values = ( #(#values),* );
                    
                        diesel::insert_into(#table_path::table)
                        .values(values)
                        .returning((#(#table_path::#fields),*))
                        .get_result::<Self::Model>(con)
                        .expect("Insert of factory failed")
                    }
                },
                _ => quote! {
                    let values = ( #(#values),* );

                    diesel::insert_into(#table_path::table)
                    .values(values)
                    .get_result::<Self::Model>(con)
                    .expect("Insert of factory failed")
                }
            }
        };

        quote! {
            impl <#lifetime> diesel_factories::Factory for #factory <#lifetime> {
                type Model = #model_type;
                type Id = #id_type;
                type Connection = #connection_type;

                fn insert(self, con: &mut Self::Connection) -> Self::Model {
                    use diesel::prelude::*;
                    #insert_code
                }

                fn id_for_model(model: &Self::Model) -> &Self::Id {
                    &model.#id_name
                }
            }
        }
    }

    fn no_fields(&self) -> bool {
        self.fields.is_empty() && self.associations.is_empty()
    }

    fn field_builder_methods(&self) -> TokenStream {
        let factory_name = &self.factory_name;

        let methods = self.fields.iter().map(|(field_name, ty)| {
            quote! {
                #[allow(missing_docs, dead_code)]
                pub fn #field_name(mut self, new: impl std::convert::Into<#ty>) -> Self {
                    self.#field_name = new.into();
                    self
                }
            }
        });

        let lifetime = &self.lifetime;

        quote! {
            impl <#lifetime> #factory_name <#lifetime> {
                #(#methods)*
            }
        }
    }

    fn association_builder_methods(&self) -> TokenStream {
        let factory_name = &self.factory_name;

        self.associations.iter().map(|(field_name, association_type, _)| {
            let association_name = format_ident!("{}", field_name.to_string().to_camel_case());
            let trait_name = format_ident!("Set{}On{}", association_name, factory_name);

            let lifetime = &association_type.lifetime;

            let model_type = &association_type.model_type;
            let other_factory = &association_type.factory_type;

            let model_impl = if association_type.is_optional {
                quote! {
                    impl<#lifetime> #trait_name<std::option::Option<& #lifetime #model_type>> for #factory_name<#lifetime> {
                        fn #field_name(mut self, t: std::option::Option<& #lifetime #model_type>) -> Self {
                            self.#field_name = t.map(diesel_factories::Association::new_model);
                            self
                        }
                    }
                }
            } else {
                quote! {
                    impl<#lifetime> #trait_name<& #lifetime #model_type> for #factory_name<#lifetime> {
                        fn #field_name(mut self, t: & #lifetime #model_type) -> Self {
                            self.#field_name = diesel_factories::Association::new_model(t);
                            self
                        }
                    }
                }
            };

            let factory_impl = if association_type.is_optional {
                quote! {
                    impl<#lifetime> #trait_name<std::option::Option<#other_factory>> for #factory_name<#lifetime> {
                        fn #field_name(mut self, t: std::option::Option<#other_factory>) -> Self {
                            self.#field_name = t.map(diesel_factories::Association::new_factory);
                            self
                        }
                    }
                }
            } else {
                quote! {
                    impl<#lifetime> #trait_name<#other_factory> for #factory_name<#lifetime> {
                        fn #field_name(mut self, t: #other_factory) -> Self {
                            self.#field_name = diesel_factories::Association::new_factory(t);
                            self
                        }
                    }
                }
            };

            quote! {
                #[allow(missing_docs, dead_code)]
                pub trait #trait_name<T> {
                    fn #field_name(self, t: T) -> Self;
                }

                #model_impl
                #factory_impl
            }
        }).collect()
    }
}

#[derive(Debug)]
struct AssociationType {
    span: Span,
    lifetime: Lifetime,
    model_type: Type,
    factory_type: Type,
    is_optional: bool,
}

impl AssociationType {
    fn new(ty: Type) -> syn::Result<Self> {
        let type_path = match ty {
            Type::Path(ty) => ty,
            _ => return Err(syn::Error::new(ty.span(), "Expected type path")),
        };

        let whole_span = type_path.span();

        if type_path.qself.is_some() {
            return Err(syn::Error::new(
                type_path.span(),
                "Qualified self types are not allowed here",
            ));
        }

        let segments = type_path.path.segments;
        let segments_span = segments.span();

        let (segments, is_optional) = peel_option(segments);
        let mut segments_iter = segments.into_iter().peekable();

        // skip fully qualified path
        let first = segments_iter
            .peek()
            .ok_or_else(|| syn::Error::new(segments_span, "Empty type path"))?;
        if first.ident == "diesel_factories" {
            segments_iter.next().ok_or_else(|| {
                syn::Error::new(
                    segments_span,
                    "Expected something after `diesel_factories::`",
                )
            })?;
        }

        let path_segment = segments_iter
            .next()
            .ok_or_else(|| syn::Error::new(segments_span, "Type path too short"))?;
        let arguments = if path_segment.ident == "Association" {
            path_segment.arguments
        } else {
            return Err(syn::Error::new(
                path_segment.span(),
                format!(
                    "Unexpected name `{}`. Expected `Association` or `diesel_factories::Association`",
                    path_segment.ident,
                )
            ));
        };

        let arguments = match arguments {
            syn::PathArguments::AngleBracketed(args) => args,
            syn::PathArguments::Parenthesized(inner) => {
                return Err(syn::Error::new(
                    inner.span(),
                    "Unexpected parenthesized type arguments. Expected angle bracketed arguments like `<...>`",
                ));
            }
            syn::PathArguments::None => {
                return Err(syn::Error::new(
                    whole_span,
                    "Missing association type arguments",
                ));
            }
        };

        if let Some(colon2) = &arguments.colon2_token {
            return Err(syn::Error::new(colon2.span(), "Unexpected `::`"));
        }

        let args_span = arguments.span();
        let mut args_iter = arguments.args.into_iter();

        let lifetime = match args_iter.next() {
            Some(inner) => match inner {
                syn::GenericArgument::Lifetime(lt) => lt,
                _ => {
                    return Err(syn::Error::new(
                        args_span,
                        "Expected generic lifetime argument",
                    ));
                }
            },
            None => {
                return Err(syn::Error::new(args_span, "Missing generic type arguments"));
            }
        };

        let model_type = match args_iter.next() {
            Some(inner) => match inner {
                syn::GenericArgument::Type(ty) => ty,
                _ => {
                    return Err(syn::Error::new(args_span, "Expected generic type argument"));
                }
            },
            None => {
                return Err(syn::Error::new(args_span, "Missing generic type arguments"));
            }
        };

        let factory_type = match args_iter.next() {
            Some(inner) => match inner {
                syn::GenericArgument::Type(ty) => ty,
                _ => {
                    return Err(syn::Error::new(args_span, "Expected generic type argument"));
                }
            },
            None => {
                return Err(syn::Error::new(args_span, "Missing generic type arguments"));
            }
        };

        if let Some(next) = args_iter.next() {
            return Err(syn::Error::new(next.span(), "Too many generic arguments"));
        }

        Ok(AssociationType {
            span: whole_span,
            lifetime,
            model_type,
            factory_type,
            is_optional,
        })
    }
}

fn peel_option(
    segments: Punctuated<PathSegment, Token![::]>,
) -> (Punctuated<PathSegment, Token![::]>, bool) {
    let original_segments = segments.clone();

    let things_inside_option = (move || {
        let mut iter = segments.into_iter();

        let first_segment = iter.next()?;

        let option_segment = if first_segment.ident == "std" && !has_path_arguments(&first_segment)
        {
            let option_module_segment = iter.next()?;
            if option_module_segment.ident == "option"
                || !has_path_arguments(&option_module_segment)
            {
                iter.next()?
            } else {
                return None;
            }
        } else if first_segment.ident == "Option" || has_path_arguments(&first_segment) {
            first_segment
        } else {
            return None;
        };

        let args = match option_segment.arguments {
            PathArguments::AngleBracketed(args) => args,
            _ => return None,
        };
        if args.colon2_token.is_some() {
            return None;
        }
        let mut args = args.args.into_iter();

        let ty = match args.next()? {
            GenericArgument::Type(ty) => ty,
            _ => return None,
        };
        if args.next().is_some() {
            return None;
        }
        let ty_path = match ty {
            Type::Path(path) => path,
            _ => return None,
        };
        if ty_path.qself.is_some() {
            println!("whoop");
            return None;
        }

        Some(ty_path.path.segments)
    })();

    if let Some(inner) = things_inside_option {
        (inner, true)
    } else {
        (original_segments, false)
    }
}

fn has_path_arguments(path_segment: &PathSegment) -> bool {
    match &path_segment.arguments {
        PathArguments::None => false,
        PathArguments::AngleBracketed(_) => true,
        PathArguments::Parenthesized(_) => true,
    }
}

impl Parse for AssociationType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<Type>()?;
        AssociationType::new(ty)
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn is_association_type_true() {
        let tokens = quote! { Association<'a, Country, CountryFactory> };
        let ty = syn::parse2::<AssociationType>(tokens).unwrap();

        assert_eq!(ty.lifetime.ident, "a");
        assert_eq!(ty.model_type, syn::parse2(quote! { Country }).unwrap());
        assert_eq!(
            ty.factory_type,
            syn::parse2(quote! { CountryFactory }).unwrap()
        );
        assert_eq!(ty.is_optional, false);
    }

    #[test]
    fn is_association_type_true_qualified() {
        let tokens = quote! { diesel_factories::Association<'b, Country, CountryFactory> };
        let ty = syn::parse2::<AssociationType>(tokens).unwrap();

        assert_eq!(ty.lifetime.ident, "b");
        assert_eq!(ty.model_type, syn::parse2(quote! { Country }).unwrap());
        assert_eq!(
            ty.factory_type,
            syn::parse2(quote! { CountryFactory }).unwrap()
        );
        assert_eq!(ty.is_optional, false);
    }

    #[test]
    fn is_association_type_true_optional() {
        let tokens = quote! { Option<Association<'a, Country, CountryFactory>> };
        let ty = syn::parse2::<AssociationType>(tokens).unwrap();

        assert_eq!(ty.lifetime.ident, "a");
        assert_eq!(ty.model_type, syn::parse2(quote! { Country }).unwrap());
        assert_eq!(
            ty.factory_type,
            syn::parse2(quote! { CountryFactory }).unwrap()
        );
        assert_eq!(ty.is_optional, true);
    }

    #[test]
    fn is_association_type_true_qualified_optional() {
        let tokens = quote! { Option<diesel_factories::Association<'b, Country, CountryFactory>> };
        let ty = syn::parse2::<AssociationType>(tokens).unwrap();

        assert_eq!(ty.lifetime.ident, "b");
        assert_eq!(ty.model_type, syn::parse2(quote! { Country }).unwrap());
        assert_eq!(
            ty.factory_type,
            syn::parse2(quote! { CountryFactory }).unwrap()
        );
        assert_eq!(ty.is_optional, true);
    }

    #[test]
    fn is_association_type_true_qualified_optional_qualified_option_also() {
        let tokens = quote! {
            std::option::Option<diesel_factories::Association<'b, Country, CountryFactory>>
        };
        let ty = syn::parse2::<AssociationType>(tokens).unwrap();

        assert_eq!(ty.lifetime.ident, "b");
        assert_eq!(ty.model_type, syn::parse2(quote! { Country }).unwrap());
        assert_eq!(
            ty.factory_type,
            syn::parse2(quote! { CountryFactory }).unwrap()
        );
        assert_eq!(ty.is_optional, true);
    }

    #[test]
    fn is_association_type_false() {
        let tokens = quote! { Country };
        let ty = syn::parse2::<AssociationType>(tokens);
        assert!(ty.is_err());
    }

    #[test]
    fn is_association_type_too_few_of_generic_args() {
        let tokens = quote! { Association<'a, Country> };
        let ty = syn::parse2::<AssociationType>(tokens);
        assert!(ty.is_err());
    }

    #[test]
    fn is_association_type_too_many_generic_args() {
        let tokens = quote! { Association<'a, Country, CountryFactory, i32> };
        let ty = syn::parse2::<AssociationType>(tokens);
        assert!(ty.is_err());
    }
}
