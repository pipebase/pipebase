use super::meta::{ContextStoreMeta, ErrorHandlerMeta, PipeMeta};
use crate::constants::{
    BOOTSTRAP_PIPE_CHANNEL_DEFAULT_BUFFER, CHANNEL_MACRO, CONTEXT_STORE_MACRO,
    ERROR_HANDLER_CHANNEL_DEFAULT_BUFFER, ERROR_HANDLER_CHANNEL_DEFAULT_TYPE,
    ERROR_HANDLER_DEFAULT_IDENT, ERROR_HANDLER_DEFAULT_RX, ERROR_HANDLER_DEFAULT_TX,
    ERROR_HANDLER_MACRO, JOIN_PIPES_MACRO, RUN_CONTEXT_STORE_MACRO, RUN_ERROR_HANDLER_MACRO,
    RUN_PIPE_MACRO, SUBSCRIBE_ERROR_HANDLER_MACRO,
};

pub trait VisitPipeMeta: Default {
    fn visit(&mut self, meta: &PipeMeta);
}

pub trait Expr {
    fn get_lhs(&self) -> Option<String> {
        None
    }
    fn get_rhs(&self) -> Option<String> {
        None
    }
    fn get_expr(&self) -> Option<String> {
        match (self.get_lhs(), self.get_rhs()) {
            (Some(lhs), Some(rhs)) => Some(format!("let {} = {}", lhs, rhs)),
            (Some(lhs), _) => Some(lhs),
            (_, Some(rhs)) => Some(rhs),
            (None, None) => None,
        }
    }
    fn as_mut(ident: &str) -> String {
        format!("mut {}", ident)
    }
    fn to_owned(ident: &str) -> String {
        format!("{}.to_owned()", ident)
    }
}

#[derive(Default)]
pub struct ChannelExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for ChannelExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let channel_ty = match meta.get_upstream_output_type_name() {
            Some(upstream_output_type_name) => upstream_output_type_name,
            None => return,
        };
        let pipe_name = meta.get_name();
        let tx_name = Self::gen_sender_name(&pipe_name);
        let rx_name = Self::gen_receiver_name(&pipe_name);
        self.lhs = Some(format!("({}, {})", tx_name, rx_name));
        self.rhs = Some(format!(
            "{}({}, {})",
            CHANNEL_MACRO, channel_ty, BOOTSTRAP_PIPE_CHANNEL_DEFAULT_BUFFER
        ));
    }
}

impl Expr for ChannelExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl ChannelExpr {
    pub fn gen_sender_name(pipe_name: &str) -> String {
        format!("tx_{}", pipe_name)
    }

    pub fn gen_receiver_name(pipe_name: &str) -> String {
        format!("rx_{}", pipe_name)
    }
}

#[derive(Default)]
pub struct PipeExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for PipeExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let pipe_name = meta.get_name();
        let ty = meta.get_ty();
        let rhs = format!(r#"{}("{}")"#, Self::pipe_type_macro(&ty), pipe_name,);
        self.lhs = Some(Self::as_mut(&pipe_name));
        self.rhs = Some(rhs);
    }
}

impl Expr for PipeExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl PipeExpr {
    fn pipe_type_macro(ty: &str) -> String {
        format!("{}!", ty)
    }
}

#[derive(Default)]
pub struct RunPipeExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for RunPipeExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let pipe_name = meta.get_name();
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        let upstream_output_type_name = meta.get_upstream_output_type_name();
        let downstream_pipe_names = meta.get_downstream_names();
        let senders_expr = Self::gen_senders_expr(downstream_pipe_names);
        // reveiver is optional for source pipe
        let rhs = match upstream_output_type_name {
            Some(_) => {
                let receiver_expr = Self::gen_recevier_expr(&pipe_name);
                format!(
                    r#"{}({}, {}, "{}", {}, {})"#,
                    RUN_PIPE_MACRO, pipe_name, config_ty, config_path, senders_expr, receiver_expr
                )
            }
            None => format!(
                r#"{}({}, {}, "{}", {})"#,
                RUN_PIPE_MACRO, pipe_name, config_ty, config_path, senders_expr
            ),
        };
        self.lhs = Some(pipe_name.to_owned());
        self.rhs = Some(rhs);
    }
}

impl Expr for RunPipeExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl RunPipeExpr {
    fn gen_recevier_expr(pipe_name: &str) -> String {
        ChannelExpr::gen_receiver_name(pipe_name)
    }

    fn gen_senders_expr(pipe_names: &Vec<String>) -> String {
        let mut sender_exprs: Vec<String> = vec![];
        for pipe_name in pipe_names {
            let sender_exp = ChannelExpr::gen_sender_name(pipe_name);
            sender_exprs.push(Self::to_owned(&sender_exp))
        }
        format!("[{}]", sender_exprs.join(", "))
    }
}

#[derive(Default)]
pub struct JoinExpr {
    pipe_names: Vec<String>,
    cstore_names: Vec<String>,
    error_handler_name: Option<String>,
}

impl VisitPipeMeta for JoinExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        self.pipe_names.push(meta.get_name().to_owned());
    }
}

impl VisitContextStoreMeta for JoinExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        self.cstore_names.push(meta.get_name().to_owned())
    }
}

impl VisitErrorHandlerMeta for JoinExpr {
    fn visit(&mut self, _meta: &ErrorHandlerMeta) {
        self.error_handler_name = Some(ERROR_HANDLER_DEFAULT_IDENT.to_owned())
    }
}

impl Expr for JoinExpr {
    fn get_expr(&self) -> Option<String> {
        let mut all_names = vec![];
        all_names.extend(self.pipe_names.to_owned());
        all_names.extend(self.cstore_names.to_owned());
        match self.error_handler_name {
            Some(ref name) => all_names.push(name.to_owned()),
            None => (),
        };
        let all_exprs = format!("{}([{}])", JOIN_PIPES_MACRO, all_names.join(","));
        Some(all_exprs)
    }
}

pub trait VisitContextStoreMeta: Default {
    fn visit(&mut self, meta: &ContextStoreMeta);
}

#[derive(Default)]
pub struct ContextStoreExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitContextStoreMeta for ContextStoreExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        let context_store_name = meta.get_name();
        let rhs = format!(r#"{}("{}")"#, CONTEXT_STORE_MACRO, context_store_name);
        self.lhs = Some(Self::as_mut(&context_store_name));
        self.rhs = Some(rhs);
    }
}

impl Expr for ContextStoreExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

#[derive(Default)]
pub struct RunContextStoreExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitContextStoreMeta for RunContextStoreExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        let name = meta.get_name();
        let pipe_exprs = meta.get_pipes().join(",");
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        let rhs = format!(
            r#"{}({}, {}, "{}", [{}])"#,
            RUN_CONTEXT_STORE_MACRO, name, config_ty, config_path, pipe_exprs
        );
        self.lhs = Some(name.to_owned());
        self.rhs = Some(rhs);
    }
}

impl Expr for RunContextStoreExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

pub trait VisitErrorHandlerMeta: Default {
    fn visit(&mut self, meta: &ErrorHandlerMeta);
}

#[derive(Default)]
pub struct ErrorChannelExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl Expr for ErrorChannelExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl VisitErrorHandlerMeta for ErrorChannelExpr {
    fn visit(&mut self, _meta: &ErrorHandlerMeta) {
        self.lhs = Some(format!(
            "({}, {})",
            ERROR_HANDLER_DEFAULT_TX, ERROR_HANDLER_DEFAULT_RX
        ));
        self.rhs = Some(format!(
            "{}({}, {})",
            CHANNEL_MACRO, ERROR_HANDLER_CHANNEL_DEFAULT_TYPE, ERROR_HANDLER_CHANNEL_DEFAULT_BUFFER
        ));
    }
}

#[derive(Default)]
pub struct SubscribeErrorExpr {
    pub rhs: Option<String>,
}

impl VisitErrorHandlerMeta for SubscribeErrorExpr {
    fn visit(&mut self, meta: &ErrorHandlerMeta) {
        let pipe_exprs = meta.get_pipes().join(",");
        let rhs = format!(
            "{}([{}], {})",
            SUBSCRIBE_ERROR_HANDLER_MACRO, pipe_exprs, ERROR_HANDLER_DEFAULT_TX
        );
        self.rhs = Some(rhs);
    }
}

impl Expr for SubscribeErrorExpr {
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

#[derive(Default)]
pub struct ErrorHandlerExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl Expr for ErrorHandlerExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl VisitErrorHandlerMeta for ErrorHandlerExpr {
    fn visit(&mut self, _meta: &ErrorHandlerMeta) {
        self.lhs = Some(Self::as_mut(ERROR_HANDLER_DEFAULT_IDENT));
        self.rhs = Some(format!("{}()", ERROR_HANDLER_MACRO));
    }
}

#[derive(Default)]
pub struct RunErrorHandlerExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl Expr for RunErrorHandlerExpr {
    fn get_lhs(&self) -> Option<String> {
        self.lhs.to_owned()
    }
    fn get_rhs(&self) -> Option<String> {
        self.rhs.to_owned()
    }
}

impl VisitErrorHandlerMeta for RunErrorHandlerExpr {
    fn visit(&mut self, meta: &ErrorHandlerMeta) {
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        let rhs = format!(
            r#"{}({}, {}, "{}", {})"#,
            RUN_ERROR_HANDLER_MACRO,
            ERROR_HANDLER_DEFAULT_IDENT,
            config_ty,
            config_path,
            ERROR_HANDLER_DEFAULT_RX
        );
        self.lhs = Some(ERROR_HANDLER_DEFAULT_IDENT.to_owned());
        self.rhs = Some(rhs);
    }
}
