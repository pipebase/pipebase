mod bootstrap;
mod constants;
mod field;
mod filter;
mod hashkey;
mod orderkey;
mod pipemeta;
mod project;
mod utils;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Project, attributes(project, input))]
pub fn derive_project(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref attributes = tokens.attrs;
    let ref data = tokens.data;
    let ref generics = tokens.generics;
    let expanded = project::impl_project(ident, attributes, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Filter, attributes(filter))]
pub fn derive_filter(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref attributes = tokens.attrs;
    let ref generics = tokens.generics;
    let expanded = filter::impl_filter(ident, attributes, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(FieldAccept, attributes(visit))]
pub fn derive_field_accept(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref data = tokens.data;
    let ref generics = tokens.generics;
    let expanded = field::impl_field_visit(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(HashKey, attributes(hkey))]
pub fn derive_hashkey(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref data = tokens.data;
    let ref generics = tokens.generics;
    let expanded = hashkey::impl_hashkey(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(OrderKey, attributes(okey))]
pub fn derive_orderkey(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref data = tokens.data;
    let ref generics = tokens.generics;
    let expanded = orderkey::impl_orderkey(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Bootstrap, attributes(pipe))]
pub fn derive_bootstrap(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ref tokens = parse_macro_input!(_tokens as DeriveInput);
    let ref ident = tokens.ident;
    let ref attributes = tokens.attrs;
    let ref generics = tokens.generics;
    let expanded = bootstrap::impl_bootstrap(ident, attributes, generics);
    proc_macro::TokenStream::from(expanded)
}
