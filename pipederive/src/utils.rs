use crate::constants::{
    PIPEMETA_FLAGS_ENV, PIPEMETA_FLAGS_SEP, PIPEMETA_FLAG_SKIP_NON_EXISTS_PIPE,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{Attribute, ItemFn, Lit, Meta, MetaList, MetaNameValue, NestedMeta};
use syn::{Data, Field, Fields, FieldsNamed};

// traversal tree with full path
pub fn find_meta_value_by_meta_path(full_path: &[&str], meta: &Meta) -> Option<Lit> {
    assert!(full_path.len() > 0);
    // reach last segment of full path
    if full_path.len() == 1 {
        if let Meta::NameValue(MetaNameValue {
            ref path, ref lit, ..
        }) = meta
        {
            let last_segment = full_path[0];
            let path = path.get_ident().unwrap().to_string();
            if path.eq(last_segment) {
                // we find the meta
                return Some(lit.clone());
            }
            return None;
        }
        return None;
    }
    if let Meta::List(ref meta_list) = meta {
        let expected_path = full_path[0];
        let actual_path = meta_list.path.get_ident().unwrap().to_string();
        if !actual_path.eq(expected_path) {
            return None;
        }
        let ref nested_metas = meta_list.nested;
        for nested_meta in nested_metas.into_iter() {
            if let NestedMeta::Meta(ref nested) = nested_meta {
                match find_meta_value_by_meta_path(&full_path[1..], nested) {
                    Some(lit) => return Some(lit),
                    None => continue,
                }
            }
        }
        return None;
    }
    return None;
}

pub fn is_meta_with_prefix(prefix_path: &[&str], meta: &Meta) -> bool {
    assert!(prefix_path.len() > 0);
    if prefix_path.len() == 1 {
        let last_segment = prefix_path[0];
        match meta {
            Meta::Path(ref path) => {
                let path = path.get_ident().unwrap().to_string();
                if path.eq(last_segment) {
                    // we find the meta
                    return true;
                }
                return false;
            }
            Meta::NameValue(MetaNameValue { ref path, .. }) => {
                let path = path.get_ident().unwrap().to_string();
                if path.eq(last_segment) {
                    // we find the meta
                    return true;
                }
                return false;
            }
            Meta::List(MetaList { ref path, .. }) => {
                let path = path.get_ident().unwrap().to_string();
                if path.eq(last_segment) {
                    // we find the meta
                    return true;
                }
                return false;
            }
        }
    }
    if let Meta::List(ref meta_list) = meta {
        let expected_path = prefix_path[0];
        let actual_path = meta_list.path.get_ident().unwrap().to_string();
        if !actual_path.eq(expected_path) {
            return false;
        }
        let ref nested_metas = meta_list.nested;
        for nested_meta in nested_metas.into_iter() {
            if let NestedMeta::Meta(ref nested) = nested_meta {
                if is_meta_with_prefix(&prefix_path[1..], nested) {
                    return true;
                }
            }
        }
        return false;
    }
    return false;
}

pub fn parse_lit_as_string(lit: &Lit) -> Option<String> {
    if let Lit::Str(lit) = lit {
        return Some(lit.value());
    }
    return None;
}

pub fn parse_lit_as_number(lit: &Lit) -> Option<String> {
    if let Lit::Int(lit) = lit {
        return Some(lit.base10_digits().to_owned());
    }
    return None;
}

pub fn get_any_attribute_by_meta_prefix(
    prefix: &str,
    attributes: &Vec<Attribute>,
    is_required: bool,
) -> Option<Attribute> {
    let prefix_path = prefix.split(".").collect::<Vec<&str>>();
    for attribute in attributes {
        if is_meta_with_prefix(&prefix_path, &attribute.parse_meta().unwrap()) {
            return Some(attribute.clone());
        }
    }
    if is_required {
        panic!("attribute not found based on prefix {}", prefix)
    }
    return None;
}

pub fn get_all_attributes_by_meta_prefix(
    prefix: &str,
    attributes: &Vec<Attribute>,
) -> Vec<Attribute> {
    let prefix_path = prefix.split(".").collect::<Vec<&str>>();
    let mut hits: Vec<Attribute> = vec![];
    for attribute in attributes {
        if is_meta_with_prefix(&prefix_path, &attribute.parse_meta().unwrap()) {
            hits.push(attribute.clone());
        }
    }
    hits
}

pub fn get_meta_string_value_by_meta_path(
    full_path: &str,
    meta: &Meta,
    is_required: bool,
) -> Option<String> {
    let ref full_path_vec = full_path.split(".").collect::<Vec<&str>>();
    if let Some(ref lit) = find_meta_value_by_meta_path(full_path_vec, meta) {
        if let Some(value) = parse_lit_as_string(lit) {
            return Some(value);
        }
    }
    if is_required {
        panic!(
            "string value for full path {} not found in attribute",
            full_path
        )
    }
    None
}

pub fn get_meta_number_value_by_meta_path(
    full_path: &str,
    attribute: &Attribute,
    is_required: bool,
) -> Option<String> {
    let ref full_path_vec = full_path.split(".").collect::<Vec<&str>>();
    if let Some(ref lit) =
        find_meta_value_by_meta_path(full_path_vec, &attribute.parse_meta().unwrap())
    {
        if let Some(value) = parse_lit_as_number(lit) {
            return Some(value);
        }
    }
    if is_required {
        panic!(
            "string value for full path {} not found in attribute",
            full_path
        )
    }
    None
}

pub fn get_type_name_token(meta: &Meta, type_name_meta_path: &str) -> TokenStream {
    let type_literal = get_meta_string_value_by_meta_path(type_name_meta_path, meta, true).unwrap();
    resolve_type_name_token(&type_literal)
}

pub fn resolve_type_name_token(type_literal: &str) -> proc_macro2::TokenStream {
    type_literal.parse().unwrap()
}

/// Resolve dotted field ident
pub fn resolve_field_path_token(field_path: &str) -> TokenStream {
    let field_idents = field_path.split(".").map(resolve_ident);
    quote! {
        #(#field_idents).*
    }
}

pub fn resolve_module_path_token(module_path: &str) -> TokenStream {
    let module_idents = module_path.split("::").map(resolve_ident);
    quote! {
        #(#module_idents)::*
    }
}

pub fn resolve_ident(field: &str) -> Ident {
    Ident::new(field, Span::call_site())
}

pub fn resolve_first_field(
    data: &Data,
    predicate: &dyn Fn(&Field) -> bool,
    required: bool,
) -> Option<Field> {
    let fields = resolve_all_fields(data, predicate);
    let field = fields.into_iter().next();
    match field {
        Some(field) => Some(field),
        None => {
            if required {
                panic!("required field not found")
            }
            None
        }
    }
}

pub fn resolve_all_fields(data: &Data, predicate: &dyn Fn(&Field) -> bool) -> Vec<Field> {
    let data_struct = match *data {
        Data::Struct(ref data_struct) => data_struct,
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    };
    let fields = match data_struct.fields {
        Fields::Named(ref fields) => find_all_fields(fields, predicate),
        Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
    };
    fields
}

pub fn find_all_fields<'a>(
    fields: &'a FieldsNamed,
    predicate: &dyn Fn(&Field) -> bool,
) -> Vec<Field> {
    fields
        .named
        .iter()
        .filter(|&field| predicate(field))
        .map(|f| f.to_owned())
        .collect()
}

pub fn get_last_stmt_span(function: &ItemFn) -> (Span, Span) {
    let mut last_stmt = function
        .block
        .stmts
        .last()
        .map(ToTokens::into_token_stream)
        .unwrap_or_default()
        .into_iter();
    let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
    let end = last_stmt.last().map_or(start, |t| t.span());
    (start, end)
}

pub fn get_meta(attribute: &Attribute) -> Meta {
    attribute.parse_meta().unwrap()
}

pub fn is_skip_non_exists_pipe() -> bool {
    let flags = match std::env::var(PIPEMETA_FLAGS_ENV) {
        Ok(flags) => flags,
        _ => return false,
    };
    let flag_set: HashSet<&str> = flags.split(PIPEMETA_FLAGS_SEP).collect();
    flag_set.contains(PIPEMETA_FLAG_SKIP_NON_EXISTS_PIPE)
}
