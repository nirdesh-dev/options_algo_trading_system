use crate::domain::constants::{IRON_BUTTERFLY_KERNEL_INDEX, IRON_CONDOR_KERNEL_INDEX};
use crate::domain::{EntryTimeMinutes, StopLossMultiplier, WingWidthPoints};

#[derive(Debug, Clone, Copy)]
pub enum StrategyType {
    IronCondor,
    IronButterfly,
}

impl StrategyType {
    pub fn to_kernel_flag(self) -> i32 {
        match self {
            StrategyType::IronCondor => IRON_CONDOR_KERNEL_INDEX,
            StrategyType::IronButterfly => IRON_BUTTERFLY_KERNEL_INDEX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StrategyParams {
    entry_time: EntryTimeMinutes,
    wing_width: WingWidthPoints,
    stop_loss: StopLossMultiplier,
}

impl StrategyParams {
    pub fn new(
        entry_time: EntryTimeMinutes,
        wing_width: WingWidthPoints,
        stop_loss: StopLossMultiplier,
    ) -> Self {
        Self {
            entry_time,
            wing_width,
            stop_loss,
        }
    }

    pub fn entry_time(&self) -> u16 {
        self.entry_time.value()
    }

    pub fn wing_width(&self) -> f32 {
        self.wing_width.value()
    }

    pub fn stop_loss(&self) -> f32 {
        self.stop_loss.value()
    }
}
