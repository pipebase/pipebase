use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Generics};

use crate::constants::{FILTER, FILTER_ALIAS, FILTER_ALIAS_DEFAULT, FILTER_PREDICATE};
use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path,
};

pub fn impl_filter(ident: &Ident, attributes: &[Attribute], generics: &Generics) -> TokenStream {
    let ident_location = ident.to_string();
    let attribute = &get_filter_attribute(attributes, &ident_location);
    let predicate = get_filter_predicate(attribute, &ident_location);
    let alias = get_filter_alias(attribute);
    let do_filter = impl_do_filter(ident, &alias, &predicate);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Filter<#ident> for #ident #type_generics #where_clause {
            fn filter(item: &#ident) -> bool {
                let do_filter = #do_filter;
                do_filter(item)
            }
        }
    }
}

fn impl_do_filter(ident: &Ident, alias: &str, predicate: &str) -> TokenStream {
    let expression: TokenStream = predicate.parse().unwrap();
    let alias_ident = Ident::new(alias, Span::call_site());
    quote! {
        | #alias_ident: &#ident | -> bool { #expression }
    }
}

fn get_filter_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(FILTER, attributes, true, ident_location).unwrap()
}

fn get_filter_alias(attribute: &Attribute) -> String {
    match get_meta_string_value_by_meta_path(FILTER_ALIAS, &get_meta(attribute), false, "") {
        Some(alias) => alias,
        None => FILTER_ALIAS_DEFAULT.to_owned(),
    }
}

fn get_filter_predicate(attribute: &Attribute, ident_location: &str) -> String {
    get_meta_string_value_by_meta_path(FILTER_PREDICATE, &get_meta(attribute), true, ident_location)
        .unwrap()
}
