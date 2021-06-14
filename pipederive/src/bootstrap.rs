use crate::pipemeta::{ChannelExpr, PipeExpr, PipeMetas};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Generics};

pub fn impl_bootstrap(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    generics: &Generics,
) -> TokenStream {
    let exprs = resolve_all_exprs(attributes);
    let concated_exprs = exprs.join(";\n");
    let expr_tokens = parse_exprs(&exprs);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Bootstrap for #ident #type_generics #where_clause {
            fn print() {
                let exprs = #concated_exprs;
                println!("{}", exprs)
            }

            fn run() {
                // #expr_tokens
            }
        }
    }
}

fn resolve_all_exprs(attributes: &Vec<Attribute>) -> Vec<String> {
    let mut all_exprs: Vec<String> = vec![];
    all_exprs.extend(resolve_channel_exprs(attributes));
    all_exprs.extend(resolve_pipe_exprs(attributes));
    all_exprs
}

fn resolve_channel_exprs(attributes: &Vec<Attribute>) -> Vec<String> {
    let metas = PipeMetas::parse(attributes);
    metas.generate_exprs::<ChannelExpr>()
}

fn resolve_pipe_exprs(attributes: &Vec<Attribute>) -> Vec<String> {
    let metas = PipeMetas::parse(attributes);
    metas.generate_exprs::<PipeExpr>()
}

fn parse_exprs(exprs: &Vec<String>) -> TokenStream {
    let expr_tokens = exprs.iter().map(|expr| parse_expr(expr));
    quote! {
        #(#expr_tokens);*
    }
}

fn parse_expr(expr: &str) -> TokenStream {
    expr.parse().unwrap()
}
