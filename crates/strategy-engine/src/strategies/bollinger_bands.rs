use crate::traits::Strategy;
use domain::{Quote, Signal};
use std::collections::HashMap;

pub struct BollingerBandsStrategy {
    id: String,
    symbols: Vec<String>,
    price_history: HashMap<String, Vec<f64>>,
    period: usize,
    std_dev_factor: f64,
}

impl BollingerBandsStrategy {
    pub fn new(id: String, symbols: Vec<String>, period: usize, std_dev_factor: f64) -> Self {
        Self {
            id,
            symbols,
            price_history: HashMap::new(),
            period,
            std_dev_factor,
        }
    }
}

impl Strategy for BollingerBandsStrategy {
    fn process_quote(&mut self, quote: &Quote) -> Option<Signal> {
        let mid_price = (quote.bid + quote.ask) / 2.0;

        // Add price to history
        let prices = self
            .price_history
            .entry(quote.symbol.clone())
            .or_insert_with(Vec::new);
        prices.push(mid_price);

        // Keep only what we need
        if prices.len() > self.period * 2 {
            prices.drain(0..self.period);
        }

        // Need enough data to calculate bands
        if prices.len() < self.period {
            return Some(Signal::Hold {
                symbol: quote.symbol.clone(),
            });
        }

        // Calculate Bollinger Bands
        let recent_prices = &prices[prices.len() - self.period..];
        let sma: f64 = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;

        let variance: f64 = recent_prices
            .iter()
            .map(|price| (price - sma).powi(2))
            .sum::<f64>()
            / recent_prices.len() as f64;
        let std_dev = variance.sqrt();

        let upper_band = sma + (self.std_dev_factor * std_dev);
        let lower_band = sma - (self.std_dev_factor * std_dev);

        println!(
            "SMA: {}, Std Dev: {}, Lower Band: {}, Upper Band: {}, Current Price: {}",
            sma, std_dev, lower_band, upper_band, mid_price
        );

        // Generate signals
        if mid_price <= lower_band {
            Some(Signal::Buy {
                symbol: quote.symbol.clone(),
                price: mid_price,
                confidence: 0.8,
            })
        } else if mid_price >= upper_band {
            Some(Signal::Sell {
                symbol: quote.symbol.clone(),
                price: mid_price,
                confidence: 0.8,
            })
        } else {
            Some(Signal::Hold {
                symbol: quote.symbol.clone(),
            })
        }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_required_symbols(&self) -> &[String] {
        &self.symbols
    }

    fn reset(&mut self) {
        self.price_history.clear();
    }
}
