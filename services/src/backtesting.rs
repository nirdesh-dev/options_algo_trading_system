use anyhow::{Context, Ok, Result, bail};
use domain::domain::DomainRules;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum ComputeMode {
    CPU { threads: usize },
    GPU { cuda_device: i32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Period(u16);

#[derive(Debug, Clone, Copy)]
pub struct StdDevFactor(f64);

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
            "Standard deviation factor",
        )?;
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

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

#[derive(Debug, Clone, Default)]
pub struct StrategyConfigBuilder {
    periods: Option<Vec<u16>>,
    std_dev_factors: Option<Vec<f64>>,
}

// Add the builder methods
impl StrategyConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn periods(mut self, periods: Vec<u16>) -> Self {
        self.periods = Some(periods);
        self
    }

    pub fn std_dev_factors(mut self, factors: Vec<f64>) -> Self {
        self.std_dev_factors = Some(factors);
        self
    }
}

// Add strategy-specific validation
pub trait Strategy {
    type Params: StrategyParams;

    fn validate_config(
        builder: &StrategyConfigBuilder,
        rules: &DomainRules,
    ) -> Result<ValidatedConfig<Self::Params>>;

    fn simulate_strategy(
        params: &Self::Params,
        price_data: &[f32],
    ) -> Result<BacktestResult<Self::Params>>;

    fn simulate_cuda(
        params: &Self::Params,
        price_data: &[f32],
    ) -> Result<BacktestResult<Self::Params>>;
}

// Bollinger Bands implementation
#[derive(Debug, Clone)]
pub struct BollingerBandsParams {
    pub period: Period,
    pub std_dev_factor: StdDevFactor,
}

impl StrategyParams for BollingerBandsParams {
    fn to_kernel_args(&self) -> Vec<f32> {
        vec![
            self.period.value() as f32,
            self.std_dev_factor.value() as f32,
        ]
    }
}

// Bollinger Bands strategy
pub struct BollingerBandsStrategy;

impl Strategy for BollingerBandsStrategy {
    type Params = BollingerBandsParams;

    fn validate_config(
        builder: &StrategyConfigBuilder,
        rules: &DomainRules,
    ) -> Result<ValidatedConfig<Self::Params>> {
        let periods = builder.periods.as_ref().context("Periods not set")?;
        let std_devs = builder
            .std_dev_factors
            .as_ref()
            .context("Std dev factors not set")?;

        let mut validated_params = Vec::new();

        // Generate cartesian product: periods × std_dev_factors
        for &period_val in periods {
            for &std_dev_val in std_devs {
                let period = Period::new(period_val, rules)?;
                let std_dev = StdDevFactor::new(std_dev_val, rules)?;
                let param = BollingerBandsParams {
                    period,
                    std_dev_factor: std_dev,
                };
                validated_params.push(param);
            }
        }

        Ok(ValidatedConfig {
            params: validated_params,
        })
    }

    fn simulate_strategy(
        params: &Self::Params,
        price_data: &[f32],
    ) -> Result<BacktestResult<Self::Params>> {
        let period = params.period.value() as usize;
        let std_dev_factor = params.std_dev_factor.value() as f32;

        if price_data.len() < period {
            return Ok(BacktestResult {
                params: params.clone(),
                total_pnl: 0.0,
                sharpe_ratio: 0.0,
                max_drawdown: 0.0,
                num_trades: 0,
                win_rate: 0.0,
                avg_trade_duration: 0.0,
                total_return: 0.0,
                volatility: 0.0,
            });
        }

        let mut trades = Vec::new();
        let mut position = 0.0;
        let mut entry_price = 0.0;
        let mut total_pnl = 0.0;
        let mut returns = Vec::new();

        for i in period..price_data.len() {
            let window = &price_data[i - period..i];
            let current_price = price_data[i];

            // Calcaulate SMA
            let sma = window.iter().sum::<f32>() / window.len() as f32;

            // Calculate Standard deviation
            let variance = window
                .iter()
                .map(|&price| (price - sma).powi(2))
                .sum::<f32>()
                / window.len() as f32;
            let std_dev = variance.sqrt();

            // Calculate bollinger bands
            let upper_band = sma + (std_dev_factor * std_dev);
            let lower_band = sma - (std_dev_factor * std_dev);

            // Trading logic
            if position == 0.0 {
                // No position - look for entry signals
                if current_price <= lower_band {
                    // BUY signal - price hit lower band
                    position = 1.0;
                    entry_price = current_price;
                } else if current_price >= upper_band {
                    // SELL signal - price hit upper band
                    position = -1.0;
                    entry_price = current_price;
                }
            } else {
                // Have position - look for exit signals
                if position > 0.0 && current_price >= sma {
                    // Exit long position when price returns to middle (SMA)
                    let trade_return = (current_price - entry_price) / entry_price;
                    total_pnl += trade_return;
                    returns.push(trade_return);
                    trades.push((entry_price, current_price, trade_return));
                    position = 0.0;
                } else if position < 0.0 && current_price <= sma {
                    // Exit short position when price returns to middle (SMA)
                    let trade_return = (entry_price - current_price) / entry_price;
                    total_pnl += trade_return;
                    returns.push(trade_return);
                    trades.push((entry_price, current_price, trade_return));
                    position = 0.0;
                }
            }
        }
        // Calculate performance metrics
        let num_trades = trades.len() as i32;
        let win_rate = if num_trades > 0 {
            trades.iter().filter(|(_, _, ret)| *ret > 0.0).count() as f32 / num_trades as f32
        } else {
            0.0
        };

        let sharpe_ratio = if returns.len() > 1 {
            let mean_return = returns.iter().sum::<f32>() / returns.len() as f32;
            let return_std = {
                let variance = returns
                    .iter()
                    .map(|&ret| (ret - mean_return).powi(2))
                    .sum::<f32>()
                    / returns.len() as f32;
                variance.sqrt()
            };
            if return_std > 0.0 {
                mean_return / return_std
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate max drawdown (simplified)
        let mut peak = 0.0;
        let mut max_drawdown = 0.0;
        let mut running_pnl = 0.0;
        for (_, _, trade_return) in &trades {
            running_pnl += trade_return;
            if running_pnl > peak {
                peak = running_pnl;
            }
            let drawdown = peak - running_pnl;
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        let volatility = if returns.len() > 1 {
            let mean = returns.iter().sum::<f32>() / returns.len() as f32;
            let variance =
                returns.iter().map(|&ret| (ret - mean).powi(2)).sum::<f32>() / returns.len() as f32;
            variance.sqrt()
        } else {
            0.0
        };

        let result = BacktestResult {
            params: params.clone(),
            total_pnl,
            sharpe_ratio,
            max_drawdown: -max_drawdown,
            num_trades,
            win_rate,
            avg_trade_duration: 1.0,
            total_return: total_pnl,
            volatility,
        };
        println!("{:?}", &result);
        Ok(result)
    }

    fn simulate_cuda(
        params: &Self::Params,
        price_data: &[f32],
    ) -> Result<BacktestResult<Self::Params>> {
        // For now, same as CPU version
        Self::simulate_strategy(params, price_data)
    }
}

#[derive(Debug, Clone)]
pub struct BacktestResult<P: StrategyParams> {
    pub params: P,               // Generic strategy parameters
    pub total_pnl: f32,          // Total profit/loss
    pub sharpe_ratio: f32,       // Risk-adjusted return
    pub max_drawdown: f32,       // Maximum loss from peak
    pub num_trades: i32,         // Total number of trades executed
    pub win_rate: f32,           // Percentage of winning trades
    pub avg_trade_duration: f32, // Average holding time (minutes)
    pub total_return: f32,       // Total return percentage
    pub volatility: f32,         // Return volatility
}

pub struct BacktestEngine {
    compute_mode: ComputeMode,
}

impl BacktestEngine {
    pub fn new(compute_mode: ComputeMode) -> Self {
        Self { compute_mode }
    }

    pub fn run_backtest<S: Strategy>(
        &self,
        config: &ValidatedConfig<S::Params>,
        price_data: &[f32],
    ) -> Result<Vec<BacktestResult<S::Params>>> {
        let params_grid = config.generate_param_grid();
        let mut results = Vec::new();

        match &self.compute_mode {
            ComputeMode::CPU { threads: _ } => {
                for params in params_grid {
                    let result = S::simulate_strategy(&params, price_data)?;
                    results.push(result);
                }
            }
            ComputeMode::GPU { cuda_device: _ } => {
                bail!("No implementation");
            }
        }
        Ok(results)
    }
}

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

#[cfg(test)]
#[path = "tests/backtesting_test.rs"]
mod backtesting_test;
