use crate::constants::{LEFT, RIGHT};
use crate::utils::{
    get_any_attribute_by_meta_prefix, meta_not_found_in_all_fields, resolve_first_field,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

pub fn impl_left_right(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let left_field = resolve_first_field(
        data,
        &is_left_field,
        true,
        &meta_not_found_in_all_fields(LEFT, &ident.to_string()),
    )
    .unwrap();
    let right_field = resolve_first_field(
        data,
        &is_right_field,
        true,
        &meta_not_found_in_all_fields(RIGHT, &ident.to_string()),
    )
    .unwrap();
    let left_field_ident = left_field.ident;
    let left_field_ty = left_field.ty;
    let right_field_ident = right_field.ident;
    let right_field_ty = right_field.ty;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics LeftRight for #ident #type_generics #where_clause {
            type L = #left_field_ty;
            type R = #right_field_ty;

            fn left(&self) -> &Self::L {
                &self.#left_field_ident
            }

            fn right(&self) -> &Self::R {
                &self.#right_field_ident
            }

            fn as_tuple(self) -> (Self::L, Self::R) {
                (self.#left_field_ident, self.#right_field_ident)
            }
        }
    }
}

fn is_left_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(LEFT, &field.attrs, false, "").is_some()
}

fn is_right_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(RIGHT, &field.attrs, false, "").is_some()
}
