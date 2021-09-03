use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConnectionConfig {
    pub uri: String,
}
