use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::Deserialize;
use crate::error::ConfigError;

#[derive(Deserialize, Debug)]
pub struct Conf {
    pub server: Server,
    pub log: Log,
    pub node: Node,
}

#[derive(Deserialize, Debug)]
pub struct Server {
    pub listen_addr: String,
}

#[derive(Deserialize, Debug)]
pub struct Log {
    pub level: String,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub id: usize,
    pub is_leader: bool,
    pub members: HashMap<String, String>,
}

pub fn read_toml(path: String) -> Result<Conf, ConfigError>{
    let mut file = File::open(path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // let toml_value:Conf = serde_json::from_str(&content)?;
    let toml_value:Conf = toml::from_str(&content)?;

    Ok(toml_value)
}