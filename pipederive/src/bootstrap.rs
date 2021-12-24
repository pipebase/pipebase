use crate::constants::{
    BOOTSTRAP_FUNCTION, BOOTSTRAP_MODULE, BOOTSTRAP_PIPE, CONTEXT_STORE, ERROR_HANDLER,
};
use crate::pipemeta::{
    ChannelExpr, ContextStoreExpr, ContextStoreMetas, ErrorChannelExpr, ErrorHandlerExpr,
    ErrorHandlerMeta, Expr, JoinExpr, PipeExpr, PipeMetas, RunContextStoreExpr,
    RunErrorHandlerExpr, RunPipeExpr, SubscribeErrorExpr,
};
use crate::utils::{
    get_all_attributes_by_meta_prefix, get_any_attribute_by_meta_prefix, get_last_stmt_span,
    get_meta_string_value_by_meta_path, resolve_ident, resolve_module_path_token,
};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Attribute, Generics, ItemFn, NestedMeta};

pub fn impl_bootstrap(ident: &Ident, attributes: &[Attribute], generics: &Generics) -> TokenStream {
    let ident_location = ident.to_string();
    let pipe_attributes = get_all_pipe_attributes(attributes);
    let cstore_attributes = get_all_context_store_attribute(attributes);
    let error_handler_attribute = get_any_error_handler_attribute(attributes);
    // parse metas
    let pipe_metas = PipeMetas::parse(&pipe_attributes, &ident_location);
    let pipe_idents = &pipe_metas.list_pipe_ident();
    let mut cstore_metas = ContextStoreMetas::parse(&cstore_attributes, &ident_location);
    cstore_metas.add_pipes(pipe_idents.to_owned());
    let error_handler_meta =
        ErrorHandlerMeta::parse(error_handler_attribute.as_ref(), &ident_location);
    let error_handler_meta = match error_handler_meta {
        Some(mut meta) => {
            meta.set_pipes(pipe_idents.to_owned());
            Some(meta)
        }
        None => None,
    };
    // generate all exprs to print
    let all_exprs = resolve_all_exprs(&pipe_metas, &cstore_metas, error_handler_meta.as_ref());
    let all_exprs = merge_all_exprs(&all_exprs, ";\n");
    // generate pipe exprs
    let channel_exprs = resolve_channel_exprs(&pipe_metas);
    let pipe_exprs = resolve_pipe_exprs(&pipe_metas);
    let run_pipe_exprs = resolve_run_pipe_exprs(&pipe_metas);
    // generate cstore exprs
    let cstore_expr = resolve_cstore_exprs(&cstore_metas);
    let run_cstore_expr = resolve_run_cstore_exprs(&cstore_metas);
    // generate error handler exprs
    let error_channel_expr = resolve_error_channel_exprs(error_handler_meta.as_ref());
    let subscribe_error_expr = resolve_subscribe_error_exprs(error_handler_meta.as_ref());
    let error_handler_expr = resolve_error_handler_exprs(error_handler_meta.as_ref());
    let run_error_handler_expr = resolve_run_error_handler_exprs(error_handler_meta.as_ref());
    // generate join all exprs
    let join_all_expr =
        resolve_join_all_expr(&pipe_metas, &cstore_metas, error_handler_meta.as_ref());
    // generate tokens for pipe exprs
    let channel_expr_tokens = parse_exprs(&channel_exprs);
    let pipe_expr_tokens = parse_exprs(&pipe_exprs);
    let run_pipe_expr_tokens = parse_exprs(&run_pipe_exprs);
    // generate tokens for cstore exprs
    let cstore_expr_tokens = parse_exprs(&cstore_expr);
    let run_cstore_expr_tokens = parse_exprs(&run_cstore_expr);
    // generate tokens for error handling exprs
    let error_channel_tokens = parse_exprs(&error_channel_expr);
    let subscribe_error_tokens = parse_exprs(&subscribe_error_expr);
    let error_handler_tokens = parse_exprs(&error_handler_expr);
    let run_error_handler_tokens = parse_exprs(&run_error_handler_expr);
    // generate token for join all - pipe and context store
    let join_all_expr_tokens = parse_exprs(&join_all_expr);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics Bootstrap for #ident #type_generics #where_clause {
            fn print() {
                let exprs = #all_exprs;
                println!("{}", exprs)
            }

            fn bootstrap(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>> {
                #channel_expr_tokens
                ;
                #pipe_expr_tokens
                ;
                #cstore_expr_tokens
                ;
                #error_channel_tokens
                ;
                #error_handler_tokens
                ;
                #subscribe_error_tokens
                ;
                #run_error_handler_tokens
                ;
                #run_cstore_expr_tokens
                ;
                #run_pipe_expr_tokens
                ;
                let run = async move {
                    #join_all_expr_tokens
                    ;
                };
                Box::pin(run)
            }
        }
    }
}

fn merge_all_exprs(exprs: &[String], sep: &str) -> String {
    exprs.join(sep)
}

fn resolve_all_exprs(
    pipe_metas: &PipeMetas,
    cstore_metas: &ContextStoreMetas,
    error_handler_meta: Option<&ErrorHandlerMeta>,
) -> Vec<String> {
    let mut all_exprs: Vec<String> = vec![];
    all_exprs.extend(resolve_channel_exprs(pipe_metas));
    all_exprs.extend(resolve_pipe_exprs(pipe_metas));
    all_exprs.extend(resolve_cstore_exprs(cstore_metas));
    all_exprs.extend(resolve_error_channel_exprs(error_handler_meta));
    all_exprs.extend(resolve_subscribe_error_exprs(error_handler_meta));
    all_exprs.extend(resolve_error_handler_exprs(error_handler_meta));
    all_exprs.extend(resolve_run_error_handler_exprs(error_handler_meta));
    all_exprs.extend(resolve_run_cstore_exprs(cstore_metas));
    all_exprs.extend(resolve_run_pipe_exprs(pipe_metas));
    all_exprs.extend(resolve_join_all_expr(
        pipe_metas,
        cstore_metas,
        error_handler_meta,
    ));
    all_exprs
}

fn resolve_channel_exprs(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_meta_exprs::<ChannelExpr>()
}

fn resolve_pipe_exprs(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_meta_exprs::<PipeExpr>()
}

fn resolve_run_pipe_exprs(metas: &PipeMetas) -> Vec<String> {
    metas.generate_pipe_meta_exprs::<RunPipeExpr>()
}

fn resolve_cstore_exprs(metas: &ContextStoreMetas) -> Vec<String> {
    metas.generate_cstore_meta_exprs::<ContextStoreExpr>()
}

fn resolve_run_cstore_exprs(metas: &ContextStoreMetas) -> Vec<String> {
    metas.generate_cstore_meta_exprs::<RunContextStoreExpr>()
}

fn resolve_error_channel_exprs(meta: Option<&ErrorHandlerMeta>) -> Vec<String> {
    match meta {
        Some(meta) => {
            let expr = meta
                .generate_error_handler_meta_expr::<ErrorChannelExpr>()
                .expect("error channel expr not found");
            vec![expr]
        }
        None => vec![],
    }
}

fn resolve_subscribe_error_exprs(meta: Option<&ErrorHandlerMeta>) -> Vec<String> {
    match meta {
        Some(meta) => {
            let expr = meta
                .generate_error_handler_meta_expr::<SubscribeErrorExpr>()
                .expect("subscribe error expr not found");
            vec![expr]
        }
        None => vec![],
    }
}

fn resolve_error_handler_exprs(meta: Option<&ErrorHandlerMeta>) -> Vec<String> {
    match meta {
        Some(meta) => {
            let expr = meta
                .generate_error_handler_meta_expr::<ErrorHandlerExpr>()
                .expect("error handler expr not found");
            vec![expr]
        }
        None => vec![],
    }
}

fn resolve_run_error_handler_exprs(meta: Option<&ErrorHandlerMeta>) -> Vec<String> {
    match meta {
        Some(meta) => {
            let expr = meta
                .generate_error_handler_meta_expr::<RunErrorHandlerExpr>()
                .expect("run error handler expr not found");
            vec![expr]
        }
        None => vec![],
    }
}

fn resolve_join_all_expr(
    pipe_metas: &PipeMetas,
    cstore_metas: &ContextStoreMetas,
    error_handler_meta: Option<&ErrorHandlerMeta>,
) -> Vec<String> {
    let mut join_expr = JoinExpr::default();
    pipe_metas.accept(&mut join_expr);
    cstore_metas.accept(&mut join_expr);
    if let Some(meta) = error_handler_meta {
        meta.accept(&mut join_expr)
    }
    match join_expr.to_expr() {
        Some(expr) => vec![expr],
        None => vec![],
    }
}

fn parse_exprs(exprs: &[String]) -> TokenStream {
    let expr_tokens = exprs.iter().map(|expr| parse_expr(expr));
    quote! {
        #(#expr_tokens);*
    }
}

fn parse_expr(expr: &str) -> TokenStream {
    expr.parse().unwrap()
}

fn get_all_pipe_attributes(attributes: &[Attribute]) -> Vec<Attribute> {
    get_all_attributes_by_meta_prefix(BOOTSTRAP_PIPE, attributes)
}

fn get_all_context_store_attribute(attributes: &[Attribute]) -> Vec<Attribute> {
    get_all_attributes_by_meta_prefix(CONTEXT_STORE, attributes)
}

fn get_any_error_handler_attribute(attributes: &[Attribute]) -> Option<Attribute> {
    get_any_attribute_by_meta_prefix(ERROR_HANDLER, attributes, false, "")
}

pub fn impl_bootstrap_macro(_args: Vec<NestedMeta>, mut function: ItemFn) -> TokenStream {
    if function.sig.asyncness.is_none() {
        panic!("the `async` keyword is missing from the function declaration")
    }
    // convert function name as bootstrap
    function.sig.ident = resolve_ident(BOOTSTRAP_FUNCTION);
    let (_, end) = get_last_stmt_span(&function);
    let body = &function.block;
    let brace_token = function.block.brace_token;
    function.block = syn::parse2(quote_spanned! { end =>
        {
            init_tracing_subscriber();
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

pub fn impl_bootstrap_main_macro(args: Vec<NestedMeta>, mut function: ItemFn) -> TokenStream {
    if function.sig.asyncness.is_none() {
        panic!("the `async` keyword is missing from the function declaration")
    }
    let modult_path_token = find_bootstrap_module(&args);
    let bootstrap = resolve_ident(BOOTSTRAP_FUNCTION);
    let (_, end) = get_last_stmt_span(&function);
    let body = &function.block;
    let brace_token = function.block.brace_token;
    function.block = syn::parse2(quote_spanned! { end =>
        {
            #body
            #modult_path_token::#bootstrap().await;
        }
    })
    .unwrap();
    function.block.brace_token = brace_token;
    let tokio_header = quote! {
        #[tokio::main]
    };
    quote! {
        #tokio_header
        #function
    }
}

fn find_bootstrap_module(args: &[NestedMeta]) -> TokenStream {
    for arg in args {
        let meta = match arg {
            NestedMeta::Meta(meta) => meta,
            _ => continue,
        };
        match get_meta_string_value_by_meta_path(BOOTSTRAP_MODULE, meta, false, "") {
            Some(ref module_path) => return resolve_module_path_token(module_path),
            None => continue,
        }
    }
    panic!("error: bootstrap module not found at 'main'")
}
