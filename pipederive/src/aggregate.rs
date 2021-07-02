use crate::constants::{AGGREGATE_SORT, AGGREGATE_SUM};

use crate::utils::{get_any_attribute_by_meta_prefix, resolve_first_field};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics, Type};

enum AggregateKind {
    Scalar,
    Vec,
}

pub fn impl_aggregate_as(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let sum_field = resolve_first_field(data, &is_sum_field, false);
    let sort_field = resolve_first_field(data, &is_sort_field, false);
    let aggregate_as_for_sum = aggregate_as(sum_field, &AggregateKind::Scalar, ident, generics);
    let aggregate_as_for_sort = aggregate_as(sort_field, &AggregateKind::Vec, ident, generics);
    quote! {
        #aggregate_as_for_sum

        #aggregate_as_for_sort
    }
}

fn is_sum_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_SUM, &field.attrs, false).is_some()
}

fn is_sort_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_SORT, &field.attrs, false).is_some()
}

fn aggregate_as(
    field: Option<Field>,
    kind: &AggregateKind,
    ident: &Ident,
    generics: &Generics,
) -> TokenStream {
    let ref field = match field {
        Some(field) => field,
        None => return quote! {},
    };
    let aggregate_ty = aggregate_ty(&field.ty, kind);
    let aggregate_value = aggregate_value(field.ident.as_ref().unwrap(), kind);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<#aggregate_ty> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> #aggregate_ty {
                #aggregate_value
            }
        }
    }
}

fn aggregate_ty(ty: &Type, kind: &AggregateKind) -> TokenStream {
    match kind {
        AggregateKind::Scalar => quote! { #ty },
        AggregateKind::Vec => quote! { Vec<#ty> },
    }
}

fn aggregate_value(ident: &Ident, kind: &AggregateKind) -> TokenStream {
    match kind {
        AggregateKind::Scalar => quote! { self.#ident.to_owned() },
        AggregateKind::Vec => quote! { vec![self.#ident.to_owned()] },
    }
}
