use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct NodeConfig {
    pub listen: String,
    pub next: Option<String>,
}

pub fn load_node_config(path: &str) -> NodeConfig {
    let data = fs::read_to_string(path).expect("config file");
    toml::from_str(&data).expect("invalid config")
}

#[derive(Deserialize)]
pub struct ClientConfig {
    pub entry: String,
    pub middle: String,
    pub exit: String,
}

pub fn load_client_config(path: &str) -> ClientConfig {
    let data = fs::read_to_string(path).expect("config file");
    toml::from_str(&data).expect("invalid config")
}
