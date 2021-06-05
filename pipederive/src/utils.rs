use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Attribute, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

// traversal tree with full path
pub fn search_meta_value_by_meta_path(full_path: &Vec<&str>, i: usize, meta: &Meta) -> Option<Lit> {
    // reach last segment of full path
    if i == full_path.len() - 1 {
        if let Meta::NameValue(MetaNameValue {
            ref path, ref lit, ..
        }) = meta
        {
            let last_segment = full_path[i];
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
        let expected_path = full_path[i];
        let actual_path = meta_list.path.get_ident().unwrap().to_string();
        if !actual_path.eq(expected_path) {
            return None;
        }
        let ref nested_metas = meta_list.nested;
        for nested_meta in nested_metas.into_iter() {
            if let NestedMeta::Meta(ref nested) = nested_meta {
                match search_meta_value_by_meta_path(full_path, i + 1, nested) {
                    Some(lit) => return Some(lit),
                    None => continue,
                }
            }
        }
        return None;
    }
    return None;
}

pub fn is_meta_with_prefix(prefix_path: &Vec<&str>, i: usize, meta: &Meta) -> bool {
    if i == prefix_path.len() - 1 {
        let last_segment = prefix_path[i];
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
            _ => return false,
        }
    }
    if let Meta::List(ref meta_list) = meta {
        let expected_path = prefix_path[i];
        let actual_path = meta_list.path.get_ident().unwrap().to_string();
        if !actual_path.eq(expected_path) {
            return false;
        }
        let ref nested_metas = meta_list.nested;
        for nested_meta in nested_metas.into_iter() {
            if let NestedMeta::Meta(ref nested) = nested_meta {
                if is_meta_with_prefix(prefix_path, i + 1, nested) {
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

pub fn search_attribute_by_meta_prefix(
    prefix: &str,
    attributes: &Vec<Attribute>,
    is_required: bool,
) -> Option<Attribute> {
    let prefix_path = prefix.split(".").collect::<Vec<&str>>();
    for attribute in attributes {
        if is_meta_with_prefix(&prefix_path, 0, &attribute.parse_meta().unwrap()) {
            return Some(attribute.clone());
        }
    }
    if is_required {
        panic!("attribute not found based on prefix")
    }
    return None;
}

pub fn search_meta_string_value_by_meta_path(
    full_path: &str,
    attribute: &Attribute,
    is_required: bool,
) -> Option<String> {
    let ref full_path_vec = full_path.split(".").collect::<Vec<&str>>();
    if let Some(ref lit) =
        search_meta_value_by_meta_path(full_path_vec, 0, &attribute.parse_meta().unwrap())
    {
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

pub fn search_meta_number_value_by_meta_path(
    full_path: &str,
    attribute: &Attribute,
    is_required: bool,
) -> Option<String> {
    let ref full_path_vec = full_path.split(".").collect::<Vec<&str>>();
    if let Some(ref lit) =
        search_meta_value_by_meta_path(full_path_vec, 0, &attribute.parse_meta().unwrap())
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

pub fn get_schema_type_ident(
    attribute: &Attribute,
    module_meta_path: &str,
    schema_meta_path: &str,
) -> proc_macro2::TokenStream {
    let type_path = get_schema_type_path_name(attribute, module_meta_path, schema_meta_path);
    resolve_type_path_ident(type_path.as_str())
}

pub fn get_schema_type_path_name(
    attribute: &Attribute,
    module_meta_path: &str,
    schema_meta_path: &str,
) -> String {
    let module = search_meta_string_value_by_meta_path(module_meta_path, attribute, true).unwrap();
    let schema = search_meta_string_value_by_meta_path(schema_meta_path, attribute, true).unwrap();
    let type_path = format!("{}::{}", module, schema);
    type_path
}

/// Resolve type path ident
pub fn resolve_type_path_ident(type_path: &str) -> proc_macro2::TokenStream {
    let field_idents = type_path.split("::").map(resolve_field_ident);
    quote! {
        #(#field_idents)::*
    }
}

/// Resolve dotted field ident
pub fn resolve_field_path_ident(field_path: &str) -> proc_macro2::TokenStream {
    let field_idents = field_path.split(".").map(resolve_field_ident);
    quote! {
        #(#field_idents).*
    }
}

pub fn resolve_field_ident(field: &str) -> Ident {
    Ident::new(field, Span::call_site())
}
