use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, Data, Field, Fields, FieldsNamed, Generics, Type};

use crate::constants::{
    INPUT, INPUT_MODULE, INPUT_SCHEMA, PROJECT, PROJECT_ALIAS, PROJECT_ALIAS_DEFAULT, PROJECT_EXPR,
    PROJECT_FROM,
};
use crate::utils::{
    resolve_field_path_ident, resolve_type_path_ident, search_attribute_by_meta_prefix,
    search_meta_string_value_by_meta_path,
};

pub fn impl_project(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    data: &Data,
    generics: &Generics,
) -> TokenStream {
    let ref input_attribute = search_attribute_by_meta_prefix(INPUT, attributes, true).unwrap();
    let input_module =
        search_meta_string_value_by_meta_path(INPUT_MODULE, input_attribute, true).unwrap();
    let input_schema =
        search_meta_string_value_by_meta_path(INPUT_SCHEMA, input_attribute, true).unwrap();
    let input_type = format!("{}::{}", input_module, input_schema);
    let input_type_ident = resolve_type_path_ident(input_type.as_str());
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let resolved_data = resolve_data(data, input_type.as_str());
    let expanded = quote! {
      // Project trait
      impl #impl_generics Project<#input_type_ident> for #ident #type_generics #where_clause {
        fn project(from: &#input_type_ident) -> #ident {
            #ident{#resolved_data}
        }
      }
    };
    expanded
}

fn resolve_data(data: &Data, input_type: &str) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => resolve_fields_named(fields, input_type),
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

fn resolve_fields_named(fields: &FieldsNamed, input_type: &str) -> TokenStream {
    let resolved_fields = fields
        .named
        .iter()
        .map(|f| resolve_field_named(f, input_type));
    quote! {
        #(#resolved_fields),*
    }
}

fn resolve_field_named(field: &Field, input_type: &str) -> proc_macro2::TokenStream {
    let ref attributes = field.attrs;
    let ref field_ident = field.ident;
    let ref field_type = field.ty;
    let ref project_attribute = search_attribute_by_meta_prefix(PROJECT, attributes, true).unwrap();
    let project_from_field_path =
        search_meta_string_value_by_meta_path(PROJECT_FROM, project_attribute, false);
    let project_expr_closure =
        search_meta_string_value_by_meta_path(PROJECT_EXPR, project_attribute, false);
    match (project_from_field_path, project_expr_closure) {
        (Some(project_field_path), _) => {
            handle_field_project(field.span(), field_ident, &project_field_path)
        }
        (None, Some(expr_closure)) => {
            let input_alias = get_project_field_input_alias(project_attribute);
            handle_field_expr(
                field.span(),
                field_ident,
                field_type,
                &expr_closure,
                input_type,
                &input_alias,
            )
        }
        (None, None) => panic!("field require one of attributes transform(project, expr)"),
    }
}

fn handle_field_project(
    field_span: Span,
    field_ident: &Option<Ident>,
    field_path: &str,
) -> proc_macro2::TokenStream {
    let field_path_ident = resolve_field_path_ident(field_path);
    quote_spanned! {field_span =>
         #field_ident: Project::project(&from.#field_path_ident)
    }
}

// evaluate field expression
fn handle_field_expr(
    field_span: Span,
    field_ident: &Option<Ident>,
    field_type: &Type,
    expr: &str,
    input_type: &str,
    input_alias: &str,
) -> proc_macro2::TokenStream {
    let input_type_ident = resolve_type_path_ident(input_type);
    let input_alias_ident = Ident::new(input_alias, Span::call_site());
    let expression: proc_macro2::TokenStream = expr.parse().unwrap();
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

fn get_project_field_input_alias(attribute: &Attribute) -> String {
    match search_meta_string_value_by_meta_path(PROJECT_ALIAS, attribute, false) {
        Some(alias) => alias,
        None => PROJECT_ALIAS_DEFAULT.to_owned(),
    }
}
