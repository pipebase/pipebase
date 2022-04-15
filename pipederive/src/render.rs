use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Generics};

use crate::constants::{RENDER_EXPR, RENDER_POSITION, RENDER_TEMPLATE};

use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_number_value_by_meta_path,
    get_meta_string_value_by_meta_path, meta_not_found_in_all_fields, resolve_all_fields,
};

pub fn impl_render(
    ident: &Ident,
    attributes: &[Attribute],
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ident_location = ident.to_string();
    let attribute = get_any_render_template_attribute(attributes, &ident_location);
    let template = get_render_template(&attribute, &ident_location);
    let mut fields = resolve_all_fields(
        data,
        true,
        &is_render_param,
        &meta_not_found_in_all_fields(RENDER_POSITION, &ident_location),
    );
    sort_render_params(&mut fields, &ident_location);
    let parameter_tokens = resolve_render_params(&fields, &ident_location);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Render for #ident #type_generics #where_clause {
            fn render(&self) -> String {
                format!(#template, #parameter_tokens)
            }
        }
    }
}

fn get_any_render_template_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(RENDER_TEMPLATE, attributes, true, ident_location).unwrap()
}

fn get_render_template(attribute: &Attribute, ident_location: &str) -> String {
    get_meta_string_value_by_meta_path(RENDER_TEMPLATE, &get_meta(attribute), true, ident_location)
        .unwrap()
}

fn is_render_param(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(RENDER_POSITION, &field.attrs, false, "").is_some()
}

fn sort_render_params(fields: &mut [Field], ident_location: &str) {
    fields.sort_by(|f0, f1| {
        get_field_pos(f0, ident_location)
            .partial_cmp(&get_field_pos(f1, ident_location))
            .unwrap()
    })
}

fn get_field_pos(field: &Field, ident_location: &str) -> usize {
    let ident_location = format!("{}.{}", ident_location, field.ident.as_ref().unwrap(),);
    let attribute = get_pos_attribute(&field.attrs, &ident_location);
    let number = get_meta_number_value_by_meta_path(
        RENDER_POSITION,
        &get_meta(&attribute),
        true,
        &ident_location,
    )
    .unwrap();
    number
        .parse()
        .unwrap_or_else(|_| panic!("parse number '{}' failed at '{}'", number, ident_location))
}

fn get_pos_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(RENDER_POSITION, attributes, true, ident_location).unwrap()
}

fn resolve_render_params(fields: &[Field], ident_location: &str) -> TokenStream {
    let params = fields
        .iter()
        .map(|field| resolve_render_param(field, ident_location));
    quote! {
        #(#params),*
    }
}

fn resolve_render_param(field: &Field, ident_location: &str) -> TokenStream {
    let span = field.span();
    let tokens = get_render_param_tokens(field, ident_location);
    quote_spanned! { span =>
        #tokens
    }
}

fn get_render_param_tokens(field: &Field, ident_location: &str) -> TokenStream {
    let expr = get_any_render_expr(field, ident_location);
    let ident = &field.ident;
    let ty = &field.ty;
    match expr {
        Some(expr) => {
            let expression: TokenStream = expr.parse().unwrap();
            let expression_closure = quote! {
                |#ident: &#ty| { #expression }
            };
            quote! {
                {
                    let eval = #expression_closure;
                    eval(&self.#ident)
                }
            }
        }
        None => quote! { self.#ident },
    }
}

fn get_any_render_expr(field: &Field, ident_location: &str) -> Option<String> {
    let attribute = get_any_attribute_by_meta_prefix(RENDER_EXPR, &field.attrs, false, "");
    match attribute {
        Some(attribute) => {
            let ident_location = match field.ident {
                Some(ref field_ident) => format!("{}.{}", ident_location, field_ident),
                None => ident_location.to_owned(),
            };
            get_meta_string_value_by_meta_path(
                RENDER_EXPR,
                &get_meta(&attribute),
                true,
                &ident_location,
            )
        }
        None => None,
    }
}
