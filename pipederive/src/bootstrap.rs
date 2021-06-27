use crate::utils::{get_all_attributes_by_meta_prefix, get_last_stmt_span};
use crate::{
    constants::BOOTSTRAP_PIPE,
    pipemeta::{ChannelExpr, PipeExpr, PipeMetas, SpawnJoinExpr},
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Attribute, Generics, ItemFn, NestedMeta};

pub fn impl_bootstrap(
    ident: &Ident,
    attributes: &Vec<Attribute>,
    generics: &Generics,
) -> TokenStream {
    let attributes = get_all_pipe_attributes(attributes);
    // generate all exprs for print
    let metas = PipeMetas::parse(&attributes);
    let all_exprs = resolve_all_exprs(&metas);
    let joined_exprs = join_all_exprs(&all_exprs, ";\n");
    // generate exprs and tokens
    let channel_exprs = resolve_channel_exprs(&metas);
    let pipe_exprs = resolve_pipe_exprs(&metas);
    let spawn_join_exprs = resolve_spawn_join_expr(&metas);
    let channel_expr_tokens = parse_exprs(&channel_exprs);
    let pipe_expr_tokens = parse_exprs(&pipe_exprs);
    let spawn_join_expr_tokens = parse_exprs(&spawn_join_exprs);
    // pipe context
    let pipe_names = metas.list_pipe_name();
    let add_pipe_contexts = resolve_pipe_contexts(&pipe_names);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Bootstrap for #ident #type_generics #where_clause {
            fn print() {
                let exprs = #joined_exprs;
                println!("{}", exprs)
            }

            fn bootstrap(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> {
                #channel_expr_tokens
                ;
                #pipe_expr_tokens
                ;
                #add_pipe_contexts
                ;
                let run = async move {
                    #spawn_join_expr_tokens
                    ;
                };
                Box::pin(run)
            }
        }
    }
}

fn join_all_exprs(exprs: &Vec<String>, sep: &str) -> String {
    exprs.join(sep)
}

fn resolve_all_exprs(metas: &PipeMetas) -> Vec<String> {
    let mut all_exprs: Vec<String> = vec![];
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

fn resolve_pipe_contexts(pipe_names: &Vec<String>) -> TokenStream {
    let pipe_context_tokens = pipe_names
        .iter()
        .map(|pipe_name| resolve_pipe_context(pipe_name));
    quote! {
        #(#pipe_context_tokens);*
    }
}

fn resolve_pipe_context(pipe_name: &str) -> TokenStream {
    let pipe_ident = Ident::new(pipe_name, Span::call_site());
    quote! {
        self.add_pipe_context(String::from(#pipe_name), #pipe_ident.get_context())
    }
}

fn get_all_pipe_attributes(attributes: &Vec<Attribute>) -> Vec<Attribute> {
    get_all_attributes_by_meta_prefix(BOOTSTRAP_PIPE, attributes)
}

pub fn impl_bootstrap_macro(_args: Vec<NestedMeta>, mut function: ItemFn) -> TokenStream {
    if function.sig.asyncness.is_none() {
        panic!("the `async` keyword is missing from the function declaration")
    }
    let (_, end) = get_last_stmt_span(&function);
    let body = &function.block;
    let brace_token = function.block.brace_token;
    function.block = syn::parse2(quote_spanned! { end =>
        {
            let mut app = {
                #body
            };
            app.bootstrap().await;
            app
        }
    })
    .unwrap();
    function.block.brace_token = brace_token;
    quote! {
        #function
    }
}
