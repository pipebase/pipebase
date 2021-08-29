use super::{
    dependency::{default_sns_dependency, UseCrate},
    meta::{meta_to_literal, meta_value_str, meta_value_usize, Meta},
    Entity, EntityAccept, VisitEntity,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ErrorHandlerConfig {
    ty: String,
    path: Option<String>,
}

impl ErrorHandlerConfig {
    fn get_ty(&self) -> &String {
        &self.ty
    }
    fn get_path(&self) -> Option<&String> {
        self.path.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ErrorHandler {
    config: ErrorHandlerConfig,
    buffer: Option<usize>,
}

impl Entity for ErrorHandler {
    fn get_id(&self) -> String {
        "error_handler".to_owned()
    }

    fn to_literal(&self, indent: usize) -> String {
        let meta = &self.get_meta();
        meta_to_literal(meta, indent)
    }
}

impl<V: VisitEntity<Self>> EntityAccept<V> for ErrorHandler {}

impl ErrorHandler {
    fn get_meta(&self) -> Meta {
        let mut metas = vec![self.get_config_meta()];
        if let Some(meta) = self.get_channel_buffer_meta() {
            metas.push(meta)
        };
        Meta::List {
            name: "error".to_owned(),
            metas,
        }
    }

    fn get_config_meta(&self) -> Meta {
        let config_ty = self.config.get_ty();
        let config_path = self.config.get_path();
        let mut metas = vec![meta_value_str("ty", config_ty, false)];
        if let Some(path) = config_path {
            metas.push(meta_value_str("path", path, false))
        };
        Meta::List {
            name: "config".to_owned(),
            metas,
        }
    }

    pub(crate) fn get_channel_buffer_meta(&self) -> Option<Meta> {
        let buffer = match self.buffer {
            Some(ref buffer) => buffer,
            None => return None,
        };
        Some(meta_value_usize("buffer", buffer))
    }
}

impl UseCrate for ErrorHandler {
    fn get_crate(&self) -> Option<super::Dependency> {
        let config_ty = self.config.get_ty().as_str();
        match config_ty {
            "SnsPipeErrorPublisherConfig" => Some(default_sns_dependency()),
            _ => None,
        }
    }
}
