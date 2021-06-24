use super::meta::PipeMeta;
use crate::constants::{CHANNEL_MACRO, PIPE_CHANNEL_DEFAULT_BUFFER, SPAWN_JOIN_MACRO};

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
        let channel_ty = match meta.get_upstream_output_name() {
            Some(upstream_output_meta) => upstream_output_meta,
            None => return,
        };
        let pipe_name = meta.get_name();
        let tx_name = Self::gen_sender_name(&pipe_name);
        let rx_name = Self::gen_receiver_name(&pipe_name);
        self.lhs = Some(format!("({}, {})", tx_name, rx_name));
        self.rhs = Some(format!(
            "{}({}, {})",
            CHANNEL_MACRO, channel_ty, PIPE_CHANNEL_DEFAULT_BUFFER
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
        let name = meta.get_name();
        let ty = meta.get_ty();
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        let upstream_output_meta = meta.get_upstream_output_name();
        let downstream_pipe_names = meta.get_downstream_names();
        let senders_expr = Self::gen_senders_expr(downstream_pipe_names);
        let receiver_expr = match upstream_output_meta {
            Some(_) => Self::gen_recevier_expr(&name),
            None => "dummy".to_owned(),
        };
        let rhs = format!(
            r#"{}("{}", "{}", {}, {}, {})"#,
            Self::type_macro(&ty),
            name,
            config_path,
            config_ty,
            receiver_expr,
            senders_expr
        );
        self.lhs = Some(Self::as_mut(&name));
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
    fn type_macro(ty: &str) -> String {
        format!("{}!", ty)
    }

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
pub struct SpawnJoinExpr {
    pub pipe_names: Vec<String>,
    pub expr: Option<String>,
}

impl VisitPipeMeta for SpawnJoinExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        self.pipe_names.push(meta.get_name().to_owned())
    }
}

impl Expr for SpawnJoinExpr {
    fn get_expr(&self) -> Option<String> {
        let pipe_names = self.pipe_names.join(", ");
        let expr = format!("{}({})", SPAWN_JOIN_MACRO, pipe_names);
        Some(expr)
    }
}
