use crate::constants::{
    CONTEXT_STORE, CONTEXT_STORE_METHOD_GET, CONTEXT_STORE_METHOD_GET_DEFAULT,
    CONTEXT_STORE_METHOD_INSERT, CONTEXT_STORE_METHOD_INSERT_DEFAULT,
};
use crate::utils::{
    get_all_attributes_by_meta_prefix, get_any_attribute_by_meta_prefix,
    get_meta_string_value_by_meta_path, resolve_first_field,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, Field, Generics};

pub fn impl_context_store(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let field = resolve_first_field(data, &is_context_store_field);
    let field_ident = field.ident;
    let attribute = get_context_store_attribute(attributes);
    let get_method_token = get_context_store_get_method_token(attribute.as_ref());
    let insert_method_token = get_context_store_insert_method_token(attribute.as_ref());
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics ContextStore for #ident #type_generics #where_clause {
            fn add_pipe_context(&mut self, pipe_name: String, context: Arc<RwLock<Context>>) {
                self.#field_ident.#insert_method_token(pipe_name, context);
            }
            fn get_pipe_context(&self, pipe_name: &str) -> Option<Arc<RwLock<Context>>> {
                match self.#field_ident.#get_method_token(pipe_name) {
                    Some(context) => Some(context.to_owned()),
                    None => None,
                }
            }
        }
    }
}

fn is_context_store_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(CONTEXT_STORE, &field.attrs, false).is_some()
}

fn get_context_store_attribute(attributes: &Vec<Attribute>) -> Option<Attribute> {
    match get_any_attribute_by_meta_prefix(CONTEXT_STORE, attributes, false) {
        Some(attribute) => Some(attribute),
        None => None,
    }
}

fn get_context_store_insert_method_token(attribute: Option<&Attribute>) -> TokenStream {
    let attribute = match attribute {
        Some(attribute) => attribute,
        None => return CONTEXT_STORE_METHOD_INSERT_DEFAULT.parse().unwrap(),
    };
    let insert_method =
        match get_meta_string_value_by_meta_path(CONTEXT_STORE_METHOD_INSERT, attribute, false) {
            Some(insert_method) => insert_method,
            None => CONTEXT_STORE_METHOD_INSERT_DEFAULT.to_owned(),
        };
    insert_method.parse().unwrap()
}

fn get_context_store_get_method_token(attribute: Option<&Attribute>) -> TokenStream {
    let attribute = match attribute {
        Some(attribute) => attribute,
        None => return CONTEXT_STORE_METHOD_GET_DEFAULT.parse().unwrap(),
    };
    let get_method =
        match get_meta_string_value_by_meta_path(CONTEXT_STORE_METHOD_GET, attribute, false) {
            Some(get_method) => get_method,
            None => CONTEXT_STORE_METHOD_GET_DEFAULT.to_owned(),
        };
    get_method.parse().unwrap()
}
