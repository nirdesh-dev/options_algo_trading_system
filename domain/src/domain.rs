use std::default;

use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: u32,
    pub ask_size: u32,
    pub bid_date: DateTime<Local>,
    pub ask_date: DateTime<Local>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Candle {
    pub symbol: Option<String>,
    // pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy {
        symbol: String,
        price: f64,
        confidence: f64,
    },
    Sell {
        symbol: String,
        price: f64,
        confidence: f64,
    },
    Hold {
        symbol: String,
    },
}

#[derive(Debug, Clone)]
pub struct TechnicalIndicatorRules {
    pub period_min: Option<u16>,
    pub period_max: Option<u16>,
    pub std_dev_min: Option<f64>,
    pub std_dev_max: Option<f64>,
    pub rsi_min: Option<u8>,
    pub rsi_max: Option<u8>,
    pub confidence_threshold_min: Option<f64>,
    pub confidence_threshold_max: Option<f64>,
}

impl Default for TechnicalIndicatorRules {
    fn default() -> Self {
        Self {
            period_min: None,
            period_max: None,
            std_dev_min: None,
            std_dev_max: None,
            rsi_min: None,
            rsi_max: None,
            confidence_threshold_min: None,
            confidence_threshold_max: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradingRules {
    pub position_size_min: Option<f64>,
    pub position_size_max: Option<f64>,
    pub stop_loss_min: Option<f64>,
    pub stop_loss_max: Option<f64>,
    pub take_profit_min: Option<f64>,
    pub take_profit_max: Option<f64>,
}

impl Default for TradingRules {
    fn default() -> Self {
        Self {
            position_size_min: None,
            position_size_max: None,
            stop_loss_min: None,
            stop_loss_max: None,
            take_profit_min: None,
            take_profit_max: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskManagementRules {
    pub max_concurrent_positions: Option<u32>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_volume: Option<u64>,
    pub max_drawdown_percent: Option<f64>,
}

impl Default for RiskManagementRules {
    fn default() -> Self {
        Self {
            max_concurrent_positions: None,
            min_price: None,
            max_price: None,
            min_volume: None,
            max_drawdown_percent: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarketHoursRules {
    pub min_holding_minutes: Option<u32>,
    pub max_holding_minutes: Option<u32>,
    pub trading_start_hour: Option<u8>,
    pub trading_end_hour: Option<u8>,
    pub exclude_first_minutes: Option<u32>,
    pub exclude_last_minutes: Option<u32>,
}

impl Default for MarketHoursRules {
    fn default() -> Self {
        Self {
            min_holding_minutes: None,
            max_holding_minutes: None,
            trading_start_hour: None,
            trading_end_hour: None,
            exclude_first_minutes: None,
            exclude_last_minutes: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomainRules {
    pub technical_indicators: TechnicalIndicatorRules,
    pub trading_rules: TradingRules,
    pub risk_management: RiskManagementRules,
    pub market_hours: MarketHoursRules,
}

impl Default for DomainRules {
    fn default() -> Self {
        Self {
            technical_indicators: TechnicalIndicatorRules::default(),
            trading_rules: TradingRules::default(),
            risk_management: RiskManagementRules::default(),
            market_hours: MarketHoursRules::default(),
        }
    }
}
