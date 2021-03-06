mod aggregate;
mod attribute;
mod bootstrap;
mod constants;
mod convert;
mod equal;
mod field;
mod filter;
mod group;
mod hashedby;
mod leftright;
mod orderedby;
mod pipemeta;
mod project;
mod render;
mod utils;

use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};

#[proc_macro_derive(Project, attributes(project))]
pub fn derive_project(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let attributes = &tokens.attrs;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = project::impl_project(ident, attributes, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Filter, attributes(filter))]
pub fn derive_filter(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let attributes = &tokens.attrs;
    let generics = &tokens.generics;
    let expanded = filter::impl_filter(ident, attributes, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(FieldAccept, attributes(visit))]
pub fn derive_field_accept(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = field::impl_field_visit(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(HashedBy, attributes(hash))]
pub fn derive_hashedby(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = hashedby::impl_hashed_by(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(OrderedBy, attributes(order))]
pub fn derive_orderby(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = orderedby::impl_ordered_by(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Bootstrap, attributes(pipe, cstore, error))]
pub fn derive_bootstrap(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let attributes = &tokens.attrs;
    let generics = &tokens.generics;
    let expanded = bootstrap::impl_bootstrap(ident, attributes, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn bootstrap(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);
    let expanded = bootstrap::impl_bootstrap_macro(args, function);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn main(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);
    let expanded = bootstrap::impl_bootstrap_main_macro(args, function);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(AggregateAs, attributes(agg))]
pub fn derive_aggregate_as(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let attributes = &tokens.attrs;
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = aggregate::impl_aggregate_as(ident, attributes, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(GroupAs, attributes(group))]
pub fn derive_group_as(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = group::impl_group_as(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Equal, attributes(equal))]
pub fn derive_equal(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = equal::impl_equal(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(LeftRight, attributes(left, right))]
pub fn derive_left_right(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = leftright::impl_left_right(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Render, attributes(render))]
pub fn derive_render(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let attributes = &tokens.attrs;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = render::impl_render(ident, attributes, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Convert, attributes(convert))]
pub fn derive_convert(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let attributes = &tokens.attrs;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = convert::impl_convert(ident, attributes, data, generics);
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(IntoAttributes, attributes(attribute))]
pub fn derive_into_attributes(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = &parse_macro_input!(_tokens as DeriveInput);
    let ident = &tokens.ident;
    let data = &tokens.data;
    let generics = &tokens.generics;
    let expanded = attribute::impl_into_attributes(ident, data, generics);
    proc_macro::TokenStream::from(expanded)
}
