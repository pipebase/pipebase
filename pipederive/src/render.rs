use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Generics};

use crate::constants::{RENDER_POSITION, RENDER_TEMPLATE};

use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_number_value_by_meta_path,
    get_meta_string_value_by_meta_path, resolve_all_fields,
};

pub fn impl_render(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let attribute = get_any_render_template_attribute(attributes);
    let template = get_render_template(&attribute);
    let mut fields = resolve_all_fields(data, &is_render_param);
    sort_render_params(&mut fields);
    let parameter_tokens = resolve_render_params(&fields);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Render for #ident #type_generics #where_clause {
            fn render(&self) -> String {
                format!(#template, #parameter_tokens)
            }
        }
    }
}

fn get_any_render_template_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(RENDER_TEMPLATE, attributes, true).unwrap()
}

fn get_render_template(attribute: &Attribute) -> String {
    get_meta_string_value_by_meta_path(RENDER_TEMPLATE, &get_meta(attribute), true).unwrap()
}

fn is_render_param(field: &Field) -> bool {
    get_any_attribute_by_meta_prefix(RENDER_POSITION, &field.attrs, false).is_some()
}

fn sort_render_params(fields: &mut Vec<Field>) {
    fields.sort_by(|f0, f1| get_field_pos(f0).partial_cmp(&get_field_pos(f1)).unwrap())
}

fn get_field_pos(field: &Field) -> usize {
    let attribute = get_pos_attribute(&field.attrs);
    let number =
        get_meta_number_value_by_meta_path(RENDER_POSITION, &get_meta(&attribute), true).unwrap();
    number
        .parse()
        .expect(&format!("parse number {} failed", number))
}

fn get_pos_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(RENDER_POSITION, attributes, true).unwrap()
}

fn resolve_render_params(fields: &Vec<Field>) -> TokenStream {
    let params = fields.into_iter().map(|field| resolve_render_param(field));
    quote! {
        #(#params),*
    }
}

fn resolve_render_param(field: &Field) -> TokenStream {
    let span = field.span();
    let ident = &field.ident;
    quote_spanned! { span =>
        self.#ident
    }
}
