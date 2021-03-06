use super::meta::{ContextStoreMeta, ErrorHandlerMeta, PipeMeta};
use crate::constants::{
    BOOTSTRAP_PIPE_CHANNELS_SUFFIX, CHANNEL_RECEIVER_SUFFIX, CHANNEL_SENDER_SUFFIX, CONFIG_SUFFIX,
    CONTEXT_COLLECTOR_IDENT_SUFFIX, ERROR_HANDLER_CHANNEL_DEFAULT_TYPE,
    ERROR_HANDLER_DEFAULT_IDENT, ERROR_HANDLER_DEFAULT_RX, ERROR_HANDLER_DEFAULT_TX, MACRO_CHANNEL,
    MACRO_COLLECT_CONTEXT, MACRO_CONFIG, MACRO_CONTEXT_STORE, MACRO_ERROR_HANDLER,
    MACRO_JOIN_PIPES, MACRO_PIPE_CHANNELS, MACRO_RUN_CONTEXT_STORE, MACRO_RUN_ERROR_HANDLER,
    MACRO_RUN_PIPE, MACRO_SUBSCRIBE_ERROR_HANDLER,
};

pub trait VisitPipeMeta: Default {
    fn visit(&mut self, meta: &PipeMeta);
}

pub trait Expr: Sized {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (None, None)
    }
    fn to_expr(self) -> Option<String> {
        match self.to_pair() {
            (Some(lhs), Some(rhs)) => Some(format!("let {} = {}", lhs, rhs)),
            (Some(lhs), _) => Some(lhs),
            (_, Some(rhs)) => Some(rhs),
            (None, None) => None,
        }
    }
    fn prepend_mut(ident: &str) -> String {
        format!("mut {}", ident)
    }
    fn append_to_owned(ident: &str) -> String {
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
        let pipe_ident = meta.get_ident();
        let tx_ident = Self::gen_sender_ident(pipe_ident);
        let rx_ident = Self::gen_receiver_ident(pipe_ident);
        let buffer = meta.get_channel_buffer();
        self.lhs = Some(format!("({}, {})", tx_ident, rx_ident));
        self.rhs = Some(format!("{}({}, {})", MACRO_CHANNEL, channel_ty, buffer));
    }
}

impl Expr for ChannelExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl ChannelExpr {
    pub fn gen_sender_ident(pipe_ident: &str) -> String {
        format!("{}{}", pipe_ident, CHANNEL_SENDER_SUFFIX)
    }

    pub fn gen_receiver_ident(pipe_ident: &str) -> String {
        format!("{}{}", pipe_ident, CHANNEL_RECEIVER_SUFFIX)
    }
}

#[derive(Default)]
pub struct PipeChannelsExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for PipeChannelsExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let pipe_ident = meta.get_ident();
        let upstream_output_type_name = meta.get_upstream_output_type_name();
        let downstream_pipe_names = meta.get_downstream_names();
        let senders_expr = Self::gen_senders_expr(downstream_pipe_names);
        // note that, receiver is none for poller and listener
        let receiver_expr = upstream_output_type_name.map(|_| Self::gen_recevier_ident(pipe_ident));
        let rhs = match receiver_expr {
            Some(receiver_expr) => format!(
                "{}({}, {})",
                MACRO_PIPE_CHANNELS, receiver_expr, senders_expr
            ),
            None => format!("{}({})", MACRO_PIPE_CHANNELS, senders_expr),
        };
        self.lhs = Some(Self::gen_ident(pipe_ident));
        self.rhs = Some(rhs);
    }
}

impl PipeChannelsExpr {
    fn gen_recevier_ident(pipe_ident: &str) -> String {
        ChannelExpr::gen_receiver_ident(pipe_ident)
    }

    fn gen_senders_expr(pipe_names: &[String]) -> String {
        let mut sender_exprs: Vec<String> = vec![];
        for pipe_name in pipe_names {
            let pipe_ident = PipeMeta::ident(pipe_name);
            let sender_exp = ChannelExpr::gen_sender_ident(&pipe_ident);
            sender_exprs.push(Self::append_to_owned(&sender_exp))
        }
        format!("[{}]", sender_exprs.join(", "))
    }

    fn gen_ident(pipe_ident: &str) -> String {
        format!("{}{}", pipe_ident, BOOTSTRAP_PIPE_CHANNELS_SUFFIX)
    }
}

impl Expr for PipeChannelsExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
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
        let pipe_ident = meta.get_ident();
        let ty = meta.get_ty();
        let rhs = format!(r#"{}("{}")"#, Self::pipe_type_macro(ty), pipe_name);
        self.lhs = Some(Self::prepend_mut(pipe_ident));
        self.rhs = Some(rhs);
    }
}

impl Expr for PipeExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl PipeExpr {
    fn pipe_type_macro(ty: &str) -> String {
        format!("{}!", ty)
    }
}

#[derive(Default)]
pub struct PipeConfigExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for PipeConfigExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let pipe_ident = meta.get_ident();
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        self.lhs = Some(Self::gen_config_ident(pipe_ident));
        self.rhs = Some(format!(
            r#"{}({}, "{}")"#,
            MACRO_CONFIG, config_ty, config_path
        ));
    }
}

impl Expr for PipeConfigExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl PipeConfigExpr {
    fn gen_config_ident(pipe_ident: &str) -> String {
        format!("{}{}", pipe_ident, CONFIG_SUFFIX)
    }
}

#[derive(Default)]
pub struct RunPipeExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for RunPipeExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let pipe_ident = meta.get_ident();
        let config_ident = PipeConfigExpr::gen_config_ident(pipe_ident);
        let pipe_channels_ident = Self::gen_ident(pipe_ident);
        let rhs = format!(
            "{}({}, {}, {})",
            MACRO_RUN_PIPE, pipe_ident, config_ident, pipe_channels_ident
        );
        self.lhs = Some(pipe_ident.to_owned());
        self.rhs = Some(rhs);
    }
}

impl Expr for RunPipeExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl RunPipeExpr {
    fn gen_ident(pipe_ident: &str) -> String {
        PipeChannelsExpr::gen_ident(pipe_ident)
    }
}

#[derive(Default)]
pub struct JoinExpr {
    pipe_idents: Vec<String>,
    cstore_idents: Vec<String>,
    error_handler_ident: Option<String>,
}

impl VisitPipeMeta for JoinExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        self.pipe_idents.push(meta.get_ident().to_owned());
    }
}

impl VisitContextStoreMeta for JoinExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        self.cstore_idents.push(meta.get_ident().to_owned())
    }
}

impl VisitErrorHandlerMeta for JoinExpr {
    fn visit(&mut self, _meta: &ErrorHandlerMeta) {
        self.error_handler_ident = Some(ERROR_HANDLER_DEFAULT_IDENT.to_owned())
    }
}

impl Expr for JoinExpr {
    fn to_expr(self) -> Option<String> {
        let mut all_idents = vec![];
        all_idents.extend(self.pipe_idents);
        all_idents.extend(self.cstore_idents);
        if let Some(ident) = self.error_handler_ident {
            all_idents.push(ident)
        };
        let all_exprs = format!("{}([{}])", MACRO_JOIN_PIPES, all_idents.join(","));
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
        let name = meta.get_name();
        let ident = meta.get_ident();
        let rhs = format!(r#"{}("{}")"#, MACRO_CONTEXT_STORE, name);
        self.lhs = Some(Self::prepend_mut(ident));
        self.rhs = Some(rhs);
    }
}

impl Expr for ContextStoreExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

#[derive(Default)]
pub struct ContextStoreConfigExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitContextStoreMeta for ContextStoreConfigExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        let cstore_ident = meta.get_ident();
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        self.lhs = Some(Self::gen_ident(cstore_ident));
        self.rhs = Some(format!(
            r#"{}({}, "{}")"#,
            MACRO_CONFIG, config_ty, config_path
        ));
    }
}

impl Expr for ContextStoreConfigExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl ContextStoreConfigExpr {
    fn gen_ident(cstore_ident: &str) -> String {
        format!("{}{}", cstore_ident, CONFIG_SUFFIX)
    }
}

#[derive(Default)]
pub struct ContextCollectorExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitContextStoreMeta for ContextCollectorExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        let ident = meta.get_ident();
        let pipe_idents = meta.get_pipes();
        self.lhs = Some(Self::gen_ident(ident));
        self.rhs = Some(format!(
            "{}([{}])",
            MACRO_COLLECT_CONTEXT,
            pipe_idents.join(",")
        ));
    }
}

impl Expr for ContextCollectorExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl ContextCollectorExpr {
    fn gen_ident(cstore_ident: &str) -> String {
        format!("{}{}", cstore_ident, CONTEXT_COLLECTOR_IDENT_SUFFIX)
    }
}

#[derive(Default)]
pub struct RunContextStoreExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitContextStoreMeta for RunContextStoreExpr {
    fn visit(&mut self, meta: &ContextStoreMeta) {
        let cstore_ident = meta.get_ident();
        let config_ident = ContextStoreConfigExpr::gen_ident(cstore_ident);
        let collector_ident = ContextCollectorExpr::gen_ident(cstore_ident);
        let rhs = format!(
            r#"{}({}, {}, {})"#,
            MACRO_RUN_CONTEXT_STORE, cstore_ident, config_ident, collector_ident
        );
        self.lhs = Some(cstore_ident.to_owned());
        self.rhs = Some(rhs);
    }
}

impl Expr for RunContextStoreExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
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
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl VisitErrorHandlerMeta for ErrorChannelExpr {
    fn visit(&mut self, meta: &ErrorHandlerMeta) {
        let buffer = meta.get_channel_buffer();
        self.lhs = Some(format!(
            "({}, {})",
            ERROR_HANDLER_DEFAULT_TX, ERROR_HANDLER_DEFAULT_RX
        ));
        self.rhs = Some(format!(
            "{}({}, {})",
            MACRO_CHANNEL, ERROR_HANDLER_CHANNEL_DEFAULT_TYPE, buffer
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
            MACRO_SUBSCRIBE_ERROR_HANDLER, pipe_exprs, ERROR_HANDLER_DEFAULT_TX
        );
        self.rhs = Some(rhs);
    }
}

impl Expr for SubscribeErrorExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (None, self.rhs)
    }
}

#[derive(Default)]
pub struct ErrorHandlerExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl Expr for ErrorHandlerExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl VisitErrorHandlerMeta for ErrorHandlerExpr {
    fn visit(&mut self, _meta: &ErrorHandlerMeta) {
        self.lhs = Some(Self::prepend_mut(ERROR_HANDLER_DEFAULT_IDENT));
        self.rhs = Some(format!("{}()", MACRO_ERROR_HANDLER));
    }
}

#[derive(Default)]
pub struct ErrorHandlerConfigExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitErrorHandlerMeta for ErrorHandlerConfigExpr {
    fn visit(&mut self, meta: &ErrorHandlerMeta) {
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        self.lhs = Some(Self::gen_ident());
        self.rhs = Some(format!(
            r#"{}({}, "{}")"#,
            MACRO_CONFIG, config_ty, config_path
        ));
    }
}

impl Expr for ErrorHandlerConfigExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl ErrorHandlerConfigExpr {
    fn gen_ident() -> String {
        format!("{}{}", ERROR_HANDLER_DEFAULT_IDENT, CONFIG_SUFFIX)
    }
}

#[derive(Default)]
pub struct RunErrorHandlerExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl Expr for RunErrorHandlerExpr {
    fn to_pair(self) -> (Option<String>, Option<String>) {
        (self.lhs, self.rhs)
    }
}

impl VisitErrorHandlerMeta for RunErrorHandlerExpr {
    fn visit(&mut self, _: &ErrorHandlerMeta) {
        let rhs = format!(
            r#"{}({}, {}, {})"#,
            MACRO_RUN_ERROR_HANDLER,
            ERROR_HANDLER_DEFAULT_IDENT,
            ErrorHandlerConfigExpr::gen_ident(),
            ERROR_HANDLER_DEFAULT_RX
        );
        self.lhs = Some(ERROR_HANDLER_DEFAULT_IDENT.to_owned());
        self.rhs = Some(rhs);
    }
}
