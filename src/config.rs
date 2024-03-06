use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConfigFile {
    pub nodes: Vec<NodeConfigEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NodeConfigEntry {
    pub node_id: String,
    pub data_endpoint: String,
    pub location: String,
}
