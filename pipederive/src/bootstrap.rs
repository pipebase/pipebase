use crate::pipemeta::{ChannelExpr, PipeExpr, PipeMetas, SpawnJoinExpr};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Generics};

pub fn impl_bootstrap(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    generics: &Generics,
) -> TokenStream {
    let exprs = resolve_all_exprs(attributes);
    let joined_exprs = join_all_exprs(&exprs, ";\n");
    let expr_tokens = parse_exprs(&exprs);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Bootstrap for #ident #type_generics #where_clause {
            fn print() {
                let exprs = #joined_exprs;
                println!("{}", exprs)
            }

            fn bootstrap() {
                // #expr_tokens
            }
        }
    }
}

fn join_all_exprs(exprs: &Vec<String>, sep: &str) -> String {
    exprs.join(sep)
}

fn resolve_all_exprs(attributes: &Vec<Attribute>) -> Vec<String> {
    let mut all_exprs: Vec<String> = vec![];
    let metas = PipeMetas::parse(attributes);
    all_exprs.extend(resolve_channel_exprs(&metas));
    all_exprs.extend(resolve_pipe_exprs(&metas));
    all_exprs.extend(resolve_spawn_join_expr(&metas));
    all_exprs
}

fn resolve_channel_exprs(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_meta_exprs::<ChannelExpr>()
}

fn resolve_pipe_exprs(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_meta_exprs::<PipeExpr>()
}

fn resolve_spawn_join_expr(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_metas_expr::<SpawnJoinExpr>()
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
