use crate::domain::constants::BLACK_SCHOLES_KERNEL_INDEX;

#[derive(Debug, Clone, Copy)]
pub enum PricingModel {
    BlackScholes,
}

impl PricingModel {
    pub fn to_kernel_flag(self) -> i32 {
        match self {
            PricingModel::BlackScholes => BLACK_SCHOLES_KERNEL_INDEX,
        }
    }
}
