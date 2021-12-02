use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, Field, Generics};

use crate::{
    constants::EQUAL,
    utils::{get_any_attribute_by_meta_prefix, meta_not_found_in_all_fields, resolve_all_fields},
};

pub fn impl_equal(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let ident_location = ident.to_string();
    let fields = resolve_all_fields(
        data,
        true,
        &is_equal_field,
        &meta_not_found_in_all_fields(EQUAL, &ident_location),
    );
    let equal_fields_token = equal_fields_token(&fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics PartialEq for #ident #type_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                #equal_fields_token
            }
        }
    }
}

fn is_equal_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(EQUAL, &field.attrs, false, "").is_some()
}

fn equal_fields_token(fields: &[Field]) -> TokenStream {
    let equal_fields = fields.iter().map(equal_field_token);
    quote! {
        #(#equal_fields)&&*
    }
}

fn equal_field_token(field: &Field) -> TokenStream {
    let field = field.to_owned();
    let field_span = field.span();
    let field_ident = field.ident.unwrap();
    quote_spanned! { field_span =>
        self.#field_ident == other.#field_ident
    }
}
