pub mod config;
pub mod error;

#[cfg(test)]
mod tests {
    use crate::config::read_toml;

    #[test]
    fn read_config() {
        let path = String::from("./config-template.toml");
        let conf = read_toml(path).unwrap();
        println!("config:{:?}", conf)
    }
}
