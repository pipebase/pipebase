use super::{
    meta::{meta_to_literal, Meta, MetaValue},
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
            Meta::Value {
                name: "name".to_owned(),
                meta: MetaValue::Str {
                    value: self.name.to_owned(),
                    raw: false,
                },
            },
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
        let mut metas = vec![Meta::Value {
            name: "ty".to_owned(),
            meta: MetaValue::Str {
                value: config_ty.to_owned(),
                raw: false,
            },
        }];
        if let Some(config_path) = config_path {
            metas.push(Meta::Value {
                name: "path".to_owned(),
                meta: MetaValue::Str {
                    value: config_path.to_owned(),
                    raw: false,
                },
            })
        };
        Meta::List {
            name: "config".to_owned(),
            metas,
        }
    }
}
