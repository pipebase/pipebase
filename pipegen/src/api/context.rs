use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ContextStore {
    name: String,
}

impl ContextStore {
    pub(crate) fn new(name: String) -> Self {
        ContextStore { name: name }
    }

    pub(crate) fn get_name(&self) -> &String {
        &self.name
    }
}
