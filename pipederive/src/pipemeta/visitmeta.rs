use super::meta::PipeMeta;
use crate::constants::{CHANNEL_MACRO, PIPE_CHANNEL_DEFAULT_BUFFER, SPAWN_JOIN_MACRO};
use core::panic;
use std::borrow::Borrow;
use std::ops::Deref;

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
}

#[derive(Default)]
pub struct ChannelExpr {
    pub lhs: Option<String>,
    pub rhs: Option<String>,
}

impl VisitPipeMeta for ChannelExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        let upstream_meta = match meta.get_upstream_meta() {
            Some(upstream_meta) => upstream_meta,
            None => return,
        };
        // if pipe has upstream, then upstream pipe must have output
        let upstream_output_meta = match upstream_meta.deref().borrow().get_output_meta() {
            Some(parent_output_meta) => parent_output_meta,
            None => panic!(
                "upstream pipe {} for {} has no output",
                upstream_meta.deref().borrow().get_name(),
                meta.deref().borrow().get_name()
            ),
        };
        let channel_ty = upstream_output_meta.get_path();
        let src_pipe_name = upstream_meta.deref().borrow().get_name();
        let dst_pipe_name = meta.get_name();
        let tx_name = Self::gen_sender_name(&src_pipe_name, &dst_pipe_name);
        let rx_name = Self::gen_receiver_name(&src_pipe_name, &dst_pipe_name);
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
    pub fn gen_sender_name(src_pipe_name: &str, dst_pipe_name: &str) -> String {
        format!("tx_{}_{}", src_pipe_name, dst_pipe_name)
    }

    pub fn gen_receiver_name(src_pipe_name: &str, dst_pipe_name: &str) -> String {
        format!("rx_{}_{}", src_pipe_name, dst_pipe_name)
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
        let kind = meta.get_kind();
        let config_meta = meta.get_config_meta();
        let config_ty = config_meta.get_ty();
        let config_path = config_meta.get_path();
        let upstream_pipe_meta = meta.get_upstream_meta();
        let downstream_pipe_metas = meta.get_downstream_metas();
        let mut downstream_pipe_names: Vec<String> = vec![];
        for downstream_pipe_meta in downstream_pipe_metas {
            let name = downstream_pipe_meta.deref().borrow().get_name();
            downstream_pipe_names.push(name)
        }
        let senders_expr = Self::gen_senders_expr(&name, downstream_pipe_names);
        let receiver_expr = match upstream_pipe_meta {
            Some(upstream_pipe_meta) => {
                let src = upstream_pipe_meta.deref().borrow().get_name();
                Self::gen_recevier_expr(&src, &name)
            }
            None => "dummy".to_owned(),
        };
        let rhs = format!(
            r#"{}("{}", "{}", {}, {}, {})"#,
            Self::kind_macro(&kind),
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
    fn kind_macro(kind: &str) -> String {
        format!("{}!", kind)
    }

    fn gen_recevier_expr(src_pipe_name: &str, dst_pipe_name: &str) -> String {
        ChannelExpr::gen_receiver_name(src_pipe_name, dst_pipe_name)
    }

    fn gen_senders_expr(src_pipe_name: &str, dst_pipe_names: Vec<String>) -> String {
        let mut sender_exprs: Vec<String> = vec![];
        for dst_pipe_name in dst_pipe_names {
            sender_exprs.push(Self::gen_sender_expr(src_pipe_name, &dst_pipe_name))
        }
        format!("[{}]", sender_exprs.join(", "))
    }

    fn gen_sender_expr(src_pipe_name: &str, dst_pipe_name: &str) -> String {
        ChannelExpr::gen_sender_name(src_pipe_name, dst_pipe_name)
    }
}

#[derive(Default)]
pub struct SpawnJoinExpr {
    pub pipe_names: Vec<String>,
    pub expr: Option<String>,
}

impl VisitPipeMeta for SpawnJoinExpr {
    fn visit(&mut self, meta: &PipeMeta) {
        self.pipe_names.push(meta.get_name())
    }
}

impl Expr for SpawnJoinExpr {
    fn get_expr(&self) -> Option<String> {
        let pipe_names = self.pipe_names.join(", ");
        let expr = format!("{}({})", SPAWN_JOIN_MACRO, pipe_names);
        Some(expr)
    }
}
