extern crate proc_macro;
extern crate proc_macro2;

use syn::{parse_macro_input, DeriveInput};

/// See the docs for "diesel_factories" for more info about this.
#[proc_macro_derive(Factory)]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    unimplemented!()
}

// fn ident(s: &str) -> Ident {
//     Ident::new(s, Span::call_site())
// }
