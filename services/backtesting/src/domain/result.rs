#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub param_set: Vec<f32>,
    pub pnl: f64,
    pub sharpe: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
}
