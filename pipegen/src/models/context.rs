use super::{
    dependency::{default_warp_dependency, UseCrate},
    meta::{meta_to_literal, meta_value_str, Meta},
    Entity, EntityAccept, VisitEntity,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ContextStoreConfig {
    ty: String,
    path: Option<String>,
}

impl ContextStoreConfig {
    fn get_ty(&self) -> &String {
        &self.ty
    }
    fn get_path(&self) -> Option<&String> {
        self.path.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ContextStore {
    name: String,
    config: ContextStoreConfig,
}

impl Entity for ContextStore {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn to_literal(&self, indent: usize) -> String {
        let meta = &self.get_meta();
        meta_to_literal(meta, indent)
    }
}

impl<V: VisitEntity<Self>> EntityAccept<V> for ContextStore {}

impl ContextStore {
    fn get_meta(&self) -> Meta {
        let metas = vec![
            meta_value_str("name", &self.name, false),
            self.get_config_meta(),
        ];
        Meta::List {
            name: "cstore".to_owned(),
            metas,
        }
    }

    fn get_config_meta(&self) -> Meta {
        let config_ty = self.config.get_ty();
        let config_path = self.config.get_path();
        let mut metas = vec![meta_value_str("ty", config_ty, false)];
        if let Some(config_path) = config_path {
            metas.push(meta_value_str("path", config_path, false))
        };
        Meta::List {
            name: "config".to_owned(),
            metas,
        }
    }
}

impl UseCrate for ContextStore {
    fn get_crate(&self) -> Option<super::Dependency> {
        let config_ty = self.config.get_ty().as_str();
        match config_ty {
            "WarpContextServerConfig" => Some(default_warp_dependency()),
            _ => None,
        }
    }
}
