use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Generics, Type};

use crate::constants::{
    PROJECT, PROJECT_ALIAS, PROJECT_ALIAS_DEFAULT, PROJECT_EXPR, PROJECT_FROM, PROJECT_INPUT,
};
use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta, get_meta_string_value_by_meta_path,
    get_type_name_token, meta_value_not_found, resolve_data, resolve_field_path_token,
};

pub fn impl_project(
    ident: &Ident,
    attributes: &[Attribute],
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ident_location = ident.to_string();
    let project_attribute = &get_any_project_attribute(attributes, &ident_location);
    let input_type_token = get_type_name_token(
        &project_attribute.parse_meta().unwrap(),
        PROJECT_INPUT,
        &ident_location,
    );
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let resolved_data = resolve_data(
        data,
        &input_type_token,
        &ident.to_string(),
        &resolve_field_named,
    );
    let expanded = quote! {
      // Project trait
      impl #impl_generics Project<#input_type_token> for #ident #type_generics #where_clause {
        fn project(from: &#input_type_token) -> #ident {
            #ident{#resolved_data}
        }
      }
    };
    expanded
}

fn resolve_field_named(
    field: &Field,
    input_type_ident: &TokenStream,
    ident_location: &str,
) -> TokenStream {
    let attributes = &field.attrs;
    let field_ident = &field.ident;
    let field_type = &field.ty;
    let project_attribute = &get_any_project_attribute(attributes, ident_location);
    let project_from = get_project_from(project_attribute);
    let project_expr = get_project_expr(project_attribute);
    if let Some(project_from) = project_from {
        return handle_project_from(field.span(), field_ident, &project_from);
    };
    if let Some(project_expr) = project_expr {
        let project_alias = get_project_alias(project_attribute);
        return handle_project_expr(
            field.span(),
            field_ident,
            field_type,
            &project_expr,
            input_type_ident,
            &project_alias,
        );
    }
    let meta_path = format!("{} or {}", PROJECT_FROM, PROJECT_EXPR);
    let ident_location = format!(
        "{}.{}",
        ident_location,
        field.ident.as_ref().unwrap().to_string()
    );
    panic!(
        "error: {}",
        meta_value_not_found(&meta_path, &ident_location)
    )
}

fn handle_project_from(
    field_span: Span,
    field_ident: &Option<Ident>,
    field_path: &str,
) -> TokenStream {
    let field_path_ident = resolve_field_path_token(field_path);
    quote_spanned! {field_span =>
         #field_ident: Project::project(&from.#field_path_ident)
    }
}

// evaluate field expression
fn handle_project_expr(
    field_span: Span,
    field_ident: &Option<Ident>,
    field_type: &Type,
    expr: &str,
    input_type_ident: &TokenStream,
    input_alias: &str,
) -> TokenStream {
    let input_alias_ident = Ident::new(input_alias, Span::call_site());
    let expression: TokenStream = expr.parse().unwrap();
    let expression_closure = quote! {
        |#input_alias_ident: &#input_type_ident| -> #field_type { #expression }
    };
    quote_spanned! {field_span =>
        #field_ident: {
            let eval = #expression_closure;
            eval(from)
        }
    }
}

fn get_project_alias(attribute: &Attribute) -> String {
    match get_meta_string_value_by_meta_path(PROJECT_ALIAS, &get_meta(attribute), false, "") {
        Some(alias) => alias,
        None => PROJECT_ALIAS_DEFAULT.to_owned(),
    }
}

fn get_any_project_attribute(attributes: &[Attribute], ident_location: &str) -> Attribute {
    get_any_attribute_by_meta_prefix(PROJECT, attributes, true, ident_location).unwrap()
}

fn get_project_from(attribute: &Attribute) -> Option<String> {
    get_meta_string_value_by_meta_path(PROJECT_FROM, &get_meta(attribute), false, "")
}

fn get_project_expr(attribute: &Attribute) -> Option<String> {
    get_meta_string_value_by_meta_path(PROJECT_EXPR, &get_meta(attribute), false, "")
}
