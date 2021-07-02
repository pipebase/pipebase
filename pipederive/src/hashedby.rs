use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, Field, Fields, FieldsNamed, Generics};

use crate::{constants::HASH, utils::get_any_attribute_by_meta_prefix};

pub fn impl_hashed_by(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let fields = resolve_field_visit(data);
    let hash_fields = hash_fields_token(&fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics std::hash::Hash for #ident #type_generics #where_clause {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #hash_fields
            }
        }
    }
}

fn resolve_field_visit(data: &Data) -> Vec<Field> {
    let data_struct = match *data {
        Data::Struct(ref data_struct) => data_struct,
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    };
    let fields = match data_struct.fields {
        Fields::Named(ref fields) => find_hash_fields(fields),
        Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
    };
    let count = fields.len();
    match count > 0 {
        true => fields,
        false => panic!("no field as hash key"),
    }
}

fn find_hash_fields(fields: &FieldsNamed) -> Vec<Field> {
    fields
        .named
        .iter()
        .filter_map(|field| hash_field(field))
        .collect()
}

fn hash_field(field: &Field) -> Option<Field> {
    match get_any_attribute_by_meta_prefix(HASH, &field.attrs, false) {
        Some(_) => Some(field.to_owned()),
        None => None,
    }
}

fn hash_fields_token(fields: &Vec<Field>) -> TokenStream {
    let hashed_fields = fields.iter().map(|f| hash_field_token(f));
    quote! {
        #(#hashed_fields);*
    }
}

fn hash_field_token(field: &Field) -> TokenStream {
    let field = field.to_owned();
    let field_span = field.span();
    let field_ident = field.ident.unwrap();
    quote_spanned! { field_span =>
        self.#field_ident.hash(state)
    }
}
