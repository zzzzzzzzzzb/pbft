use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("toml ser err: {0}")]
    TomlSerError(#[from] toml::ser::Error),
    #[error("toml de err: {0}")]
    TomlDeError(#[from] toml::de::Error),
    #[error("io error: {0}")]
    StdIOError(#[from] std::io::Error),
    // #[error("serde_json error: {0}")]
    // SerdeJsonError(#[from] serde_json::Error)
}
