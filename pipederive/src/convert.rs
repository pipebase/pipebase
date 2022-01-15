use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Generics};

use crate::constants::{CONVERT_FROM, CONVERT_INPUT};
use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path,
    get_type_name_token, resolve_data, resolve_field_path_token,
};

pub fn impl_convert(
    ident: &Ident,
    attributes: &[Attribute],
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ident_location = ident.to_string();
    let input_attribute = &get_any_input_attribute(attributes, &ident_location);
    let input_type_token =
        get_type_name_token(&get_meta(input_attribute), CONVERT_INPUT, &ident_location);
    let resolved_data = resolve_data(
        data,
        &input_type_token,
        &ident.to_string(),
        &resolve_field_named,
    );
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics Convert<#input_type_token> for #ident #type_generics #where_clause {
            fn convert(input: #input_type_token) -> Self {
                #ident{#resolved_data}
            }
        }
    };
    expanded
}

fn get_any_input_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(CONVERT_INPUT, attributes, true, ident_location).unwrap()
}

fn resolve_field_named(
    field: &Field,
    _input_type_ident: &TokenStream,
    ident_location: &str,
) -> TokenStream {
    let attributes = &field.attrs;
    let field_ident = &field.ident;
    let ident_location = format!("{}.{}", ident_location, field_ident.as_ref().unwrap(),);
    let convert_from_attribute = &get_any_convert_from_attribute(attributes, &ident_location);
    let convert_from = get_convert_from(convert_from_attribute, &ident_location);
    let field_path_ident = resolve_field_path_token(&convert_from);
    quote_spanned! {field.span() =>
         #field_ident: input.#field_path_ident.into()
    }
}

fn get_any_convert_from_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(CONVERT_FROM, attributes, true, ident_location).unwrap()
}

fn get_convert_from(attribute: &Attribute, ident_location: &str) -> String {
    get_meta_string_value_by_meta_path(CONVERT_FROM, &get_meta(attribute), true, ident_location)
        .unwrap()
}
