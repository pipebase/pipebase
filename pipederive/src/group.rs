use crate::constants::GROUP;
use crate::utils::{
    get_any_attribute_by_meta_prefix, meta_not_found_in_all_fields, resolve_first_field,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

pub fn impl_group_as(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let ident_location = ident.to_string();
    let group_field = resolve_first_field(
        data,
        &is_group_field,
        true,
        &meta_not_found_in_all_fields(GROUP, &ident_location),
    )
    .unwrap();
    let group_as_ty = group_field.ty;
    let group_field_ident = group_field.ident.unwrap();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics GroupAs<#group_as_ty> for #ident #type_generics #where_clause {
            fn group(&self) -> #group_as_ty {
                self.#group_field_ident.to_owned()
            }
        }
    }
}

fn is_group_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(GROUP, &field.attrs, false, "").is_some()
}
