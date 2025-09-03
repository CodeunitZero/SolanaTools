use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rust_log_level: String,
    pub endpoint: String,
    pub auth_token: String,
    pub pump_fun_fee_account: String,
    pub pump_fun_program: String,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let s = fs::read_to_string(path)?;
        let cfg: Self = toml::from_str(&s)?;
        Ok(cfg)
    }
}