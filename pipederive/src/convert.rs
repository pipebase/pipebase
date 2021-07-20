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
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ref input_attribute = get_any_input_attribute(attributes);
    let input_type_token = get_type_name_token(&get_meta(input_attribute), CONVERT_INPUT);
    let resolved_data = resolve_data(data, &input_type_token, &resolve_field_named);
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

fn get_any_input_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(CONVERT_INPUT, attributes, true).unwrap()
}

fn resolve_field_named(field: &Field, _input_type_ident: &TokenStream) -> TokenStream {
    let ref attributes = field.attrs;
    let ref field_ident = field.ident;
    let ref convert_from_attribute = get_any_convert_from_attribute(attributes);
    let convert_from = get_convert_from(convert_from_attribute);
    let field_path_ident = resolve_field_path_token(&convert_from);
    quote_spanned! {field.span() =>
         #field_ident: input.#field_path_ident.into()
    }
}

fn get_any_convert_from_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(CONVERT_FROM, attributes, true).unwrap()
}

fn get_convert_from(attribute: &Attribute) -> String {
    get_meta_string_value_by_meta_path(CONVERT_FROM, &get_meta(attribute), true).unwrap()
}
