use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ChainInfo {
    pub id: String,
    pub name: String,
    pub protocol: String,
}
