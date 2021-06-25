use crate::constants::ORDER_KEY;
use crate::utils::{get_any_attribute_by_meta_prefix, resolve_first_field};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

pub fn impl_orderkey(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let field = resolve_first_field(data, &is_ord_key_field);
    let field_ident = field.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Ord for #ident #type_generics #where_clause {
            fn cmp(&self, other: &Self) -> Ordering {
                self.#field_ident.cmp(&other.#field_ident)
            }
        }

        impl #impl_generics PartialOrd for #ident #type_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl #impl_generics PartialEq for #ident #type_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.#field_ident == other.#field_ident
            }
        }
    }
}

fn is_ord_key_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(ORDER_KEY, &field.attrs, false).is_some()
}
