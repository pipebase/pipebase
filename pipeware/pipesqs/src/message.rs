use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SqsMessageAttributeData {
    String(String),
    Binary(Vec<u8>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SqsMessageAttributeValue {
    pub ty: String,
    pub data: SqsMessageAttributeData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SqsMessageAttributes {
    pub values: HashMap<String, SqsMessageAttributeValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SqsMessage {
    pub body: String,
    pub message_attributes: SqsMessageAttributes,
}

impl SqsMessageAttributes {
    pub fn has_attribute(&self, name: &str, ty: &str) -> bool {
        if !self.values.contains_key(name) {
            return false;
        }
        let attribute_value = self
            .get_attribute_value(name)
            .expect(&format!("attribute {} not found", name));
        if !attribute_value.ty.eq(ty) {
            return false;
        }
        return true;
    }

    fn get_attribute_value(&self, name: &str) -> Option<&SqsMessageAttributeValue> {
        self.values.get(name)
    }

    pub fn has_attribute_string_value(&self, name: &str, ty: &str, other_data: &str) -> bool {
        if !self.has_attribute(name, ty) {
            return false;
        }
        let attribute_value = self
            .get_attribute_value(name)
            .expect(&format!("attribute {} not found", name));
        match &attribute_value.data {
            SqsMessageAttributeData::Binary(_) => false,
            SqsMessageAttributeData::String(data) => data.eq(other_data),
        }
    }

    pub fn get_attribute_string_value(&self, name: &str, ty: &str) -> Option<String> {
        if !self.has_attribute(name, ty) {
            return None;
        }
        let attribute_value = self
            .get_attribute_value(name)
            .expect(&format!("attribute {} not found", name));
        match &attribute_value.data {
            SqsMessageAttributeData::Binary(_) => None,
            SqsMessageAttributeData::String(data) => Some(data.to_owned()),
        }
    }
}
