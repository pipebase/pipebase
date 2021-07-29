use crate::constants::{
    AGGREGATE_AVG_F32, AGGREGATE_AVG_F32_DEFAULT_TYPE, AGGREGATE_COUNT32,
    AGGREGATE_COUNT32_DEFAULT_TYPE, AGGREGATE_SUM, AGGREGATE_TOP,
};

use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path,
    resolve_first_field,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, Field, Generics};

pub fn impl_aggregate_as(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let sum_field = resolve_first_field(data, &is_sum_field, false, String::new());
    let avgf32_field = resolve_first_field(data, &is_avgf32_field, false, String::new());
    let aggregate_for_sum = aggregate_for_sum(sum_field, ident, generics);
    let aggregate_for_avgf32 = aggregate_for_avgf32(avgf32_field, ident, generics);
    let aggregate_for_top = match is_top(attributes) {
        true => aggregate_for_top(ident, generics),
        false => quote! {},
    };
    let aggregate_for_count32 = match get_count32_attribute(attributes) {
        Some(ref attribute) => aggregate_for_count32(ident, generics, attribute),
        None => quote! {},
    };
    quote! {
        #aggregate_for_sum

        #aggregate_for_avgf32

        #aggregate_for_top

        #aggregate_for_count32
    }
}

fn is_sum_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_SUM, &field.attrs, false).is_some()
}

fn is_avgf32_field(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_AVG_F32, &field.attrs, false).is_some()
}

fn get_avgf32_ty(field: &Field) -> TokenStream {
    let ref attribute =
        get_any_attribute_by_meta_prefix(AGGREGATE_AVG_F32, &field.attrs, true).unwrap();
    let ty = get_meta_string_value_by_meta_path(AGGREGATE_AVG_F32, &get_meta(attribute), false);
    match ty {
        Some(ty) => ty.parse().unwrap(),
        None => AGGREGATE_AVG_F32_DEFAULT_TYPE.parse().unwrap(),
    }
}

fn is_top(attributes: &Vec<Attribute>) -> bool {
    get_any_attribute_by_meta_prefix(AGGREGATE_TOP, attributes, false).is_some()
}

fn get_count32_attribute(attributes: &Vec<Attribute>) -> Option<Attribute> {
    get_any_attribute_by_meta_prefix(AGGREGATE_COUNT32, attributes, false)
}

fn get_count32_ty(attribute: &Attribute) -> TokenStream {
    let ty = get_meta_string_value_by_meta_path(AGGREGATE_COUNT32, &get_meta(attribute), false);
    match ty {
        Some(ty) => ty.parse().unwrap(),
        None => AGGREGATE_COUNT32_DEFAULT_TYPE.parse().unwrap(),
    }
}

fn aggregate_for_sum(field: Option<Field>, ident: &Ident, generics: &Generics) -> TokenStream {
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

fn aggregate_for_top(ident: &Ident, generics: &Generics) -> TokenStream {
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<Vec<Self>> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> Vec<Self> {
                vec![self.to_owned()]
            }
        }
    }
}

fn aggregate_for_count32(ident: &Ident, generics: &Generics, attribute: &Attribute) -> TokenStream {
    let count32_ty = get_count32_ty(attribute);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<#count32_ty> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> #count32_ty {
                let count32 = Count32::new(1);
                count32.into()
            }
        }
    }
}

fn aggregate_for_avgf32(field: Option<Field>, ident: &Ident, generics: &Generics) -> TokenStream {
    let ref field = match field {
        Some(field) => field,
        None => return quote! {},
    };
    let agg_field_ident = field.ident.as_ref().unwrap();
    let avgf32_ty = get_avgf32_ty(field);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics AggregateAs<#avgf32_ty> for #ident #type_generics #where_clause {
            fn aggregate_value(&self) -> #avgf32_ty {
                let avg = Averagef32::new(self.#agg_field_ident.to_owned() as f32, 1.0);
                avg.into()
            }
        }
    }
}
