use crate::domain::{EntryTimeMinutes, StopLossMultiplier, WingWidthPoints};

#[derive(Debug, Clone, Copy)]
pub struct IronCondorParams {
    pub entry_time: EntryTimeMinutes,
    pub wing_width: WingWidthPoints,
    pub stop_loss: StopLossMultiplier,
}

impl IronCondorParams {
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
    pub fn to_tuple(&self) -> (f32, f32, f32) {
        (
            self.entry_time.value() as f32,
            self.wing_width.value(),
            self.stop_loss.value(),
        )
    }
}
