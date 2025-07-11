use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DomainRules {
    pub entry_time_min: u16,
    pub entry_time_max: u16,
    pub wing_width_min: f32,
    pub wing_width_max: f32,
    pub stop_loss_min: f32,
    pub stop_loss_max: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IronCondorConfig {
    pub entry_times: Vec<u16>,
    pub wing_widths: Vec<f32>,
    pub stop_losses: Vec<f32>,
}

#[derive(Debug, Deserialize)]
pub struct StrategyConfigs {
    pub iron_condor: IronCondorConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub domain_rules: DomainRules,
    pub strategies: StrategyConfigs,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file '{}'", path))?;
        let parsed = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse config file '{}'", path))?;
        Ok(parsed)
    }
}
