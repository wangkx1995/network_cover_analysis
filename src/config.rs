use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub source: SourceConfig,
    pub output: OutputConfig,
    pub coverage: Option<CoverageConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CoverageConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    pub source_type: String,
    pub csv: Option<CsvConfig>,
    pub postgres: Option<PostgresConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CsvConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct PostgresConfig {
    pub conn_str: String,
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub dir: String,
    pub market_direct: String,
    pub micro_market: String,
    pub poi_micro: String,
}

pub fn load_config(path: &str) -> Result<AppConfig> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}
