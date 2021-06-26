use super::meta::{Meta, MetaValue};
use super::{DataField, DataType};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct ContextStore {
    name: String,
    methods: HashMap<String, String>,
    data_ty: DataType,
}

impl ContextStore {
    pub fn new(name: String) -> Self {
        let mut methods = HashMap::new();
        methods.insert("get".to_owned(), "get".to_owned());
        methods.insert("insert".to_owned(), "insert".to_owned());
        let data_ty = DataType::HashMap {
            key_data_ty: Box::new(DataType::String),
            value_data_ty: Box::new(DataType::Object("Arc<RwLock<Context>>".to_owned())),
        };
        ContextStore {
            name: name,
            methods: methods,
            data_ty: data_ty,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_methods(&self) -> &HashMap<String, String> {
        &self.methods
    }

    fn methods_as_meta(&self) -> Meta {
        let mut method_metas = vec![];
        for (name, value) in &self.methods {
            method_metas.push(Meta::Value {
                name: name.to_owned(),
                meta: MetaValue::Str(value.to_owned()),
            });
        }
        Meta::List {
            name: "method".to_owned(),
            metas: method_metas,
        }
    }

    fn get_meta(&self) -> Meta {
        Meta::List {
            name: "cstore".to_owned(),
            metas: vec![self.methods_as_meta()],
        }
    }

    pub fn as_data_field(&self) -> DataField {
        DataField::new_named_field(
            self.data_ty.to_owned(),
            self.name.to_owned(),
            vec![self.get_meta()],
            false,
            false,
        )
    }
}
