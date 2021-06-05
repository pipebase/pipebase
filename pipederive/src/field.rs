use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Fields, FieldsNamed, Generics};

use crate::{constants::FIELD_VISIT, utils::search_attribute_by_meta_prefix};

pub fn impl_field_visit(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let field = resolve_field_visit(data);
    let field_type = field.ty;
    let field_ident = field.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics FieldAccept<#field_type> for #ident #type_generics #where_clause {
            fn accept(&self, visitor: &mut FieldVisitor<#field_type>) {
                visitor.visit(self.#field_ident.to_owned())
            }
        }
    }
}

fn resolve_field_visit(data: &Data) -> Field {
    let data_struct = match *data {
        Data::Struct(ref data_struct) => data_struct,
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    };
    let field = match data_struct.fields {
        Fields::Named(ref fields) => find_field_visit(fields),
        Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
    };
    match field {
        Some(field) => field.to_owned(),
        None => panic!("no field to visit"),
    }
}

fn find_field_visit(fields: &FieldsNamed) -> Option<&Field> {
    fields
        .named
        .iter()
        .filter(|&field| visit_field(field))
        .next()
}

fn visit_field(field: &Field) -> bool {
    search_attribute_by_meta_prefix(FIELD_VISIT, &field.attrs, false).is_some()
}
