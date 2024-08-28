use config::config::read_toml;
use consensus::members::Members;
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    let conf = match read_toml(String::from("./config.toml")) {
        Err(e) => panic!("read toml err: {}", e),
        Ok(conf) => conf,
    };

    let id_list: HashMap<usize, String> = conf
        .node
        .members
        .iter()
        // .enumerate()
        .map(|(key, value)| {
            let usize_key = key.parse().unwrap();
            (usize_key, value.clone())
        })
        .collect();

    let membership = Arc::new(Members::new(conf.node.id, conf.node.is_leader, &id_list));

    let level = tracing::Level::from_str(&conf.log.level).unwrap();

    fmt().with_max_level(level).init();

    if let Err(err) = consensus::server::run(membership.clone(), conf.server.listen_addr).await {
        panic!("{}", err)
    }
}
