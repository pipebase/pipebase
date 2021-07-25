use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SQSMessageAttributeData {
    String(String),
    Binary(Vec<u8>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SQSMessageAttributeValue {
    pub ty: String,
    pub data: SQSMessageAttributeData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SQSMessageAttributes {
    pub values: HashMap<String, SQSMessageAttributeValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SQSMessage {
    pub body: String,
    pub message_attributes: SQSMessageAttributes,
}

impl SQSMessageAttributes {
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

    fn get_attribute_value(&self, name: &str) -> Option<&SQSMessageAttributeValue> {
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
            SQSMessageAttributeData::Binary(_) => false,
            SQSMessageAttributeData::String(data) => data.eq(other_data),
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
            SQSMessageAttributeData::Binary(_) => None,
            SQSMessageAttributeData::String(data) => Some(data.to_owned()),
        }
    }
}
