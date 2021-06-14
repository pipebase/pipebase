use super::meta::PipeMeta;
use crate::constants::{CHANNEL_MACRO, PIPE_CHANNEL_DEFAULT_BUFFER};
use std::borrow::Borrow;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

pub trait VisitPipeMeta: Default {
    fn visit(&mut self, meta: &PipeMeta);
}

pub trait Expr {
    fn get_lhs(&self) -> Option<String>;
    fn get_rhs(&self) -> Option<String>;
    fn get_expr(&self) -> Option<String> {
        match (self.get_lhs(), self.get_rhs()) {
            (Some(lhs), Some(rhs)) => Some(format!("let {} = {}", lhs, rhs)),
            (Some(lhs), _) => Some(lhs),
            (_, Some(rhs)) => Some(rhs),
            (None, None) => None,
        }
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
        let parent_output_meta = upstream_meta.deref().borrow().get_output_meta();
        let channel_ty = parent_output_meta.get_path();
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
        let rhs = match upstream_pipe_meta {
            Some(upstream_pipe_meta) => {
                let src = (*upstream_pipe_meta).borrow().get_name();
                let receiver_expr = Self::gen_recevier_expr(&src, &name);
                format!(
                    r#"{}("{}", "{}", {}, {}, {})"#,
                    Self::kind_macro(&kind),
                    name,
                    config_path,
                    config_ty,
                    receiver_expr,
                    senders_expr
                )
            }
            None => format!(
                r#"{}("{}", "{}", {}, {})"#,
                Self::kind_macro(&kind),
                name,
                config_path,
                config_ty,
                senders_expr
            ),
        };
        self.lhs = Some(name);
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
pub struct SpawnJoinPipeExpr {
    // pub pipe_names: Vec<String>
    pub expr: Option<String>,
}
