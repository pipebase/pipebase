use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, Field, Generics};

use crate::{
    constants::HASH,
    utils::{get_any_attribute_by_meta_prefix, meta_not_found_in_all_fields, resolve_all_fields},
};

pub fn impl_hashed_by(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let ident_location = ident.to_string();
    let fields = resolve_all_fields(
        data,
        true,
        &is_hash_field,
        &meta_not_found_in_all_fields(HASH, &ident_location),
    );
    let hash_fields_token = hash_fields_token(&fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics std::hash::Hash for #ident #type_generics #where_clause {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                #hash_fields_token
            }
        }
    }
}

fn is_hash_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(HASH, &field.attrs, false, "").is_some()
}

fn hash_fields_token(fields: &[Field]) -> TokenStream {
    let hashed_fields = fields.iter().map(hash_field_token);
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
