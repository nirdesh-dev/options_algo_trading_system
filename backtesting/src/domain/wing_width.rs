use crate::config::DomainRules;
use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct WingWidthPoints(f32);

impl WingWidthPoints {
    pub fn new(value: f32, rules: &DomainRules) -> Result<Self> {
        if (rules.wing_width_min..=rules.wing_width_max).contains(&value) {
            Ok(Self(value))
        } else {
            bail!(
                "Invalid wing width: {}. Allowed range: {} - {} points.",
                value,
                rules.wing_width_min,
                rules.wing_width_max
            );
        }
    }
    pub fn value(&self) -> f32 {
        self.0
    }
}
