use crate::domain::strategy::StrategyParams;

#[derive(Debug, Clone, Copy)]
pub struct BacktestResult {
    pub param: StrategyParams,
    pub total_pnl: f32,
    pub sharpe: f32,
    pub drawdown: f32,
}
