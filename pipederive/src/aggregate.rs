use crate::constants::{AGGREGATE_SUM, AGGREGATE_TOP};

use crate::utils::{get_any_attribute_by_meta_prefix, resolve_first_field};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, Field, Generics};

pub fn impl_aggregate_as(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let sum_field = resolve_first_field(data, &is_sum_field, false);

    let aggregate_for_sum = aggregate_as_scalar(sum_field, ident, generics);
    let aggregate_for_sort = match is_sort(attributes) {
        true => aggregate_for_sort(ident, generics),
        false => quote! {},
    };
    quote! {
        #aggregate_for_sum

        #aggregate_for_sort
    }
}

fn is_sum_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_SUM, &field.attrs, false).is_some()
}

fn is_sort(attributes: &Vec<Attribute>) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_TOP, attributes, false).is_some()
}

fn aggregate_as_scalar(field: Option<Field>, ident: &Ident, generics: &Generics) -> TokenStream {
    let ref field = match field {
        Some(field) => field,
        None => return quote! {},
    };
    let agg_field_ty = &field.ty;
    let agg_field_ident = field.ident.as_ref().unwrap();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<#agg_field_ty> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> #agg_field_ty {
                self.#agg_field_ident.to_owned()
            }
        }
    }
}

fn aggregate_for_sort(ident: &Ident, generics: &Generics) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<Vec<Self>> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> Vec<Self> {
                vec![self.to_owned()]
            }
        }
    }
}
