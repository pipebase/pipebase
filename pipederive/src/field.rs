use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Field, Generics};

use crate::{
    constants::FIELD_VISIT,
    utils::{resolve_first_field, search_attribute_by_meta_prefix},
};

pub fn impl_field_visit(ident: &Ident, data: &Data, generics: &Generics) -> TokenStream {
    let field = resolve_first_field(data, &is_visit_field);
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

fn is_visit_field(field: &Field) -> bool {
    search_attribute_by_meta_prefix(FIELD_VISIT, &field.attrs, false).is_some()
}
