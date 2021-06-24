use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Generics};

use crate::constants::{FILTER, FILTER_ALIAS, FILTER_ALIAS_DEFAULT, FILTER_PREDICATE};
use crate::utils::{get_any_attribute_by_meta_prefix, get_meta_string_value_by_meta_path};

pub fn impl_filter(ident: &Ident, attributes: &Vec<Attribute>, generics: &Generics) -> TokenStream {
    let ref attribute = get_filter_attribute(attributes);
    let predicate = get_filter_predicate(attribute);
    let alias = get_filter_alias(attribute);
    let do_filter = impl_do_filter(ident, &alias, &predicate);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Filter<#ident> for #ident #type_generics #where_clause {
            fn filter(item: &#ident) -> Option<#ident> {
                let do_filter = #do_filter;
                match do_filter(item) {
                    true => Some(item.to_owned()),
                    false => None
                }
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

fn get_filter_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(FILTER, attributes, true).unwrap()
}

fn get_filter_alias(attribute: &Attribute) -> String {
    match get_meta_string_value_by_meta_path(FILTER_ALIAS, attribute, false) {
        Some(alias) => alias,
        None => FILTER_ALIAS_DEFAULT.to_owned(),
    }
}

fn get_filter_predicate(attribute: &Attribute) -> String {
    get_meta_string_value_by_meta_path(FILTER_PREDICATE, attribute, true).unwrap()
}
