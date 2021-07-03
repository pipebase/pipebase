use crate::constants::GROUP;
use crate::utils::{get_any_attribute_by_meta_prefix, resolve_first_field};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

pub fn impl_group_as(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let group_field = resolve_first_field(data, &is_group_field, true).unwrap();
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
    get_any_attribute_by_meta_prefix(GROUP, &field.attrs, false).is_some()
}
