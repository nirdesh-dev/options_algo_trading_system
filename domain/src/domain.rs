use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: u32,
    pub ask_size: u32,
    pub biddate: DateTime<Local>,
    pub askdate: DateTime<Local>,
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
