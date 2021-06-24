use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Fields, FieldsNamed, Generics, Type};

use crate::constants::{
    PROJECT, PROJECT_ALIAS, PROJECT_ALIAS_DEFAULT, PROJECT_EXPR, PROJECT_FROM, PROJECT_INPUT,
};
use crate::utils::{
    get_any_attribute_by_meta_prefix, get_meta_string_value_by_meta_path, get_type_name_token,
    resolve_field_path_token,
};

pub fn impl_project(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ref project_attribute = get_any_project_attribute(attributes);
    let input_type_token = get_type_name_token(project_attribute, PROJECT_INPUT);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let resolved_data = resolve_data(data, &input_type_token);
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

fn resolve_data(data: &Data, input_type_token: &TokenStream) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => resolve_fields_named(fields, input_type_token),
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

fn resolve_fields_named(fields: &FieldsNamed, input_type_token: &TokenStream) -> TokenStream {
    let resolved_fields = fields
        .named
        .iter()
        .map(|f| resolve_field_named(f, input_type_token));
    quote! {
        #(#resolved_fields),*
    }
}

fn resolve_field_named(field: &Field, input_type_ident: &TokenStream) -> TokenStream {
    let ref attributes = field.attrs;
    let ref field_ident = field.ident;
    let ref field_type = field.ty;
    let ref project_attribute = get_any_project_attribute(attributes);
    let project_from = get_project_from(project_attribute);
    let project_expr = get_project_expr(project_attribute);
    match (project_from, project_expr) {
        (Some(project_from), _) => handle_project_from(field.span(), field_ident, &project_from),
        (None, Some(project_expr)) => {
            let project_alias = get_project_alias(project_attribute);
            handle_project_expr(
                field.span(),
                field_ident,
                field_type,
                &project_expr,
                input_type_ident,
                &project_alias,
            )
        }
        (None, None) => panic!("field require one of attributes transform(project, expr)"),
    }
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
    match get_meta_string_value_by_meta_path(PROJECT_ALIAS, attribute, false) {
        Some(alias) => alias,
        None => PROJECT_ALIAS_DEFAULT.to_owned(),
    }
}

fn get_any_project_attribute(attributes: &Vec<Attribute>) -> Attribute {
    get_any_attribute_by_meta_prefix(PROJECT, attributes, true).unwrap()
}

fn get_project_from(attribute: &Attribute) -> Option<String> {
    get_meta_string_value_by_meta_path(PROJECT_FROM, attribute, false)
}

fn get_project_expr(attribute: &Attribute) -> Option<String> {
    get_meta_string_value_by_meta_path(PROJECT_EXPR, attribute, false)
}
