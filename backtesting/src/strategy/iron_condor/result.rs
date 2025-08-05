use super::params::IronCondorParams;

#[derive(Debug, Clone)]
pub struct IronCondorBacktestResult {
    pub params: IronCondorParams,
    pub total_pnl: f32,
    pub sharpe: f32,
    pub drawdown: f32,
}
