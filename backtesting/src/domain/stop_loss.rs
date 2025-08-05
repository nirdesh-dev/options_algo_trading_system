use crate::config::DomainRules;
use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct StopLossMultiplier(f32);

impl StopLossMultiplier {
    pub fn new(value: f32, rules: &DomainRules) -> Result<Self> {
        if (rules.stop_loss_min..=rules.stop_loss_max).contains(&value) {
            Ok(Self(value))
        } else {
            bail!(
                "Invalid stop loss: {}. Allowed range: {} - {} multiplier.",
                value,
                rules.stop_loss_min,
                rules.stop_loss_max
            );
        }
    }
    pub fn value(&self) -> f32 {
        self.0
    }
}
