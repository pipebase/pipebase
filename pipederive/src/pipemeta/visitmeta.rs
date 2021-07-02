use super::meta::PipeMeta;
use crate::constants::{
    BOOTSTRAP_PIPE_CHANNEL_DEFAULT_BUFFER, CHANNEL_MACRO, RUN_PIPES_MACRO, RUN_PIPE_MACRO,
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
        let receiver_expr = match upstream_output_type_name {
            Some(_) => {
                let expr = Self::gen_recevier_expr(&pipe_name);
                format!("Some({})", expr)
            }
            None => "None".to_owned(),
        };
        let expr = format!(
            r#"{}({}, {}, "{}", {}, {})"#,
            RUN_PIPE_MACRO, pipe_name, config_ty, config_path, receiver_expr, senders_expr
        );
        self.lhs = Some(pipe_name.to_owned());
        self.rhs = Some(expr);
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
pub struct RunPipesExpr {
    pipe_names: Vec<String>,
}

impl VisitPipeMeta for RunPipesExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        self.pipe_names.push(meta.get_name().to_owned());
    }
}

impl Expr for RunPipesExpr {
    fn get_expr(&self) -> Option<String> {
        let all_exprs = self.pipe_names.join(", ");
        let all_exprs = format!("{}([{}])", RUN_PIPES_MACRO, all_exprs);
        Some(all_exprs)
    }
}
