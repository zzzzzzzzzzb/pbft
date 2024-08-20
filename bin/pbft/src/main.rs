use config::config::read_toml;
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

    let level = tracing::Level::from_str(&conf.log.level).unwrap();

    fmt().with_max_level(level).init();

    if let Err(err) = consensus::server::run(
        conf.node.id,
        conf.node.is_leader,
        id_list,
        conf.server.listen_addr,
    )
    .await
    {
        panic!("{}", err)
    }
}
