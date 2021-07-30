use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, Field, Generics};

use crate::utils::resolve_data;

pub fn impl_into_attributes(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let resolved_data = resolve_data(data, &quote! {}, "", &resolve_field_named);
    let expanded = quote! {
        impl #impl_generics IntoAttributes for #ident #type_generics #where_clause {
            fn into_attributes(self) -> std::collections::HashMap<String, Value> {
                let attributes = vec![#resolved_data];
                attributes.into_iter().map(|(k, v)| (k, v)).collect()
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
    quote_spanned! {field.span() =>
         (String::from(#field_name), Value::from(self.#field_ident))
    }
}
