use crate::constants::ORDER;
use crate::utils::{
    get_any_attribute_by_meta_prefix, meta_not_found_in_all_fields, resolve_first_field,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

pub fn impl_ordered_by(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let field = resolve_first_field(
        data,
        &is_ord_field,
        true,
        meta_not_found_in_all_fields(ORDER, &ident.to_string()).into(),
    )
    .unwrap();
    let field_ident = field.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Ord for #ident #type_generics #where_clause {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.#field_ident.cmp(&other.#field_ident)
            }
        }

        impl #impl_generics std::cmp::PartialOrd for #ident #type_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    }
}

fn is_ord_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(ORDER, &field.attrs, false).is_some()
}
