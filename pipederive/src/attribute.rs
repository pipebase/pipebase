use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::Attribute;
use syn::{spanned::Spanned, Data, Field, Generics};

use crate::constants::ATTRIBUTE_ALIAS;
use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path, resolve_data,
};

pub fn impl_into_attributes(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let resolved_data = resolve_data(data, &quote! {}, "", &resolve_field_named);
    let expanded = quote! {
        impl #impl_generics IntoAttributes for #ident #type_generics #where_clause {
            fn into_attributes(self) -> std::collections::HashMap<String, Value> {
                let attributes = vec![#resolved_data];
                attributes.into_iter().map(|(k, v)| (k, v)).collect()
            }

            fn into_attribute_tuples(self) -> std::vec::Vec<(String, Value)> {
                vec![#resolved_data]
            }
        }
    };
    expanded
}

fn resolve_field_named(
    field: &Field,
    _input_type_ident: &TokenStream,
    _ident_location: &str,
) -> TokenStream {
    let field_ident = field.ident.as_ref().unwrap();
    let field_name = field_ident.to_string();
    let attribute_field_name = get_attribute_field_alias(&field.attrs).unwrap_or(field_name);
    quote_spanned! {field.span() =>
         (String::from(#attribute_field_name), Value::from(self.#field_ident))
    }
}

fn get_attribute_field_alias(attributes: &[Attribute]) -> Option<String> {
    let attribute = match get_any_attribute_by_meta_prefix(ATTRIBUTE_ALIAS, attributes, false, "") {
        Some(attribute) => attribute,
        None => return None,
    };
    get_meta_string_value_by_meta_path(ATTRIBUTE_ALIAS, &get_meta(&attribute), false, "")
}
