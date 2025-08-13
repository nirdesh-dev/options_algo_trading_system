use anyhow::{Result, bail};
use domain::domain::DomainRules;
use std::fmt::Debug;

pub trait StrategyParams: Clone + Send + Sync + Debug {
    fn to_kernel_args(&self) -> Vec<f32>;
}

pub struct ValidatedConfig<P: StrategyParams> {
    params: Vec<P>,
}

impl<P: StrategyParams> ValidatedConfig<P> {
    pub fn generate_param_grid(&self) -> Vec<P> {
        self.params.clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Period(u16);

#[derive(Debug, Clone, Copy)]
pub struct StdDevFactor(f64);

// Generic validation function
fn validate_range<T>(value: T, min: Option<T>, max: Option<T>, param_name: &str) -> Result<()>
where
    T: PartialOrd + std::fmt::Display + Copy,
{
    if let Some(min_val) = min {
        if value < min_val {
            bail!("{} {} is below minimum {}", param_name, value, min_val);
        }
    }

    if let Some(max_val) = max {
        if value > max_val {
            bail!("{} {} is above maximum {}", param_name, value, max_val);
        }
    }

    Ok(())
}

impl Period {
    pub fn new(value: u16, rules: &DomainRules) -> Result<Self> {
        let tech_rules = &rules.technical_indicators;
        validate_range(
            value,
            tech_rules.period_min,
            tech_rules.period_max,
            "Period",
        )?;
        Ok(Self(value))
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl StdDevFactor {
    pub fn new(value: f64, rules: &DomainRules) -> Result<Self> {
        let tech_rules = &rules.technical_indicators;
        validate_range(
            value,
            tech_rules.std_dev_min,
            tech_rules.std_dev_max,
            "Period",
        )?;
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
#[path = "tests/backtesting_test.rs"]
mod backtesting_test;
