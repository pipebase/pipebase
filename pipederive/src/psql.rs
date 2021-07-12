use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Generics};

use crate::constants::{PSQL, PSQL_QUERY};

use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path,
    resolve_all_fields,
};

pub fn impl_psql(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let attribute = get_any_psql_attribute(attributes);
    let query = get_psql_query(&attribute);
    let fields = resolve_all_fields(data, &is_psql_field);
    let parameter_tokens = resolve_psql_parameters(&fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Psql for #ident #type_generics #where_clause {
            fn psql(&self) -> String {
                // let query = #query;
                format!(#query, #parameter_tokens)
            }
        }
    }
}

fn get_any_psql_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(PSQL, attributes, true).unwrap()
}

fn get_psql_query(attribute: &Attribute) -> String {
    get_meta_string_value_by_meta_path(PSQL_QUERY, &get_meta(attribute), true).unwrap()
}

fn is_psql_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(PSQL, &field.attrs, false).is_some()
}

fn resolve_psql_parameters(fields: &Vec<Field>) -> TokenStream {
    let params = fields
        .into_iter()
        .map(|field| resolve_psql_parameter(field));
    quote! {
        #(#params),*
    }
}

fn resolve_psql_parameter(field: &Field) -> TokenStream {
    let span = field.span();
    let ident = &field.ident;
    quote_spanned! { span =>
        self.#ident
    }
}
