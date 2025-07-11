use crate::config::DomainRules;
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub struct EntryTimeMinutes(u16);

impl EntryTimeMinutes {
    pub fn new(value: u16, rules: &DomainRules) -> Result<Self> {
        if (rules.entry_time_min..=rules.entry_time_max).contains(&value) {
            Ok(Self(value))
        } else {
            bail!(
                "Invalid entry time: {}. Allowed range: {} - {} minutes.",
                value,
                rules.entry_time_min,
                rules.entry_time_max
            );
        }
    }
    pub fn value(&self) -> u16 {
        self.0
    }
}
