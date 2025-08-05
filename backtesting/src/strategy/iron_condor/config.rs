use serde::Deserialize;

use crate::config::DomainRules;
use crate::domain::{EntryTimeMinutes, StopLossMultiplier, WingWidthPoints};
use crate::strategy::iron_condor::params::IronCondorParams;

#[derive(Debug, Deserialize, Clone)]
pub struct IronCondorRawConfig {
    pub entry_times: Vec<u16>,
    pub wing_widths: Vec<f32>,
    pub stop_losses: Vec<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IronCondorConfig {
    pub entry_times: Vec<EntryTimeMinutes>,
    pub wing_widths: Vec<WingWidthPoints>,
    pub stop_losses: Vec<StopLossMultiplier>,
}

impl IronCondorConfig {
    pub fn from_raw(raw: IronCondorRawConfig, rules: &DomainRules) -> anyhow::Result<Self> {
        let entry_times = raw
            .entry_times
            .iter()
            .map(|&v| EntryTimeMinutes::new(v, rules))
            .collect::<anyhow::Result<_>>()?;
        let wing_widths = raw
            .wing_widths
            .iter()
            .map(|&v| WingWidthPoints::new(v, rules))
            .collect::<anyhow::Result<_>>()?;
        let stop_losses = raw
            .stop_losses
            .iter()
            .map(|&v| StopLossMultiplier::new(v, rules))
            .collect::<anyhow::Result<_>>()?;

        Ok(Self {
            entry_times,
            wing_widths,
            stop_losses,
        })
    }

    pub fn generate_param_grid(&self) -> Vec<IronCondorParams> {
        let mut grid = Vec::new();
        for &entry_time in &self.entry_times {
            for &wing_width in &self.wing_widths {
                for &stop_loss in &self.stop_losses {
                    grid.push(IronCondorParams::new(entry_time, wing_width, stop_loss));
                }
            }
        }
        grid
    }
}
