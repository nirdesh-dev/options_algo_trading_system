use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Quote, Signal};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct StrategyEngine {
    strategy: Arc<Mutex<BollingerBandsStrategy>>,
    signal_subscribers: Arc<Mutex<Vec<Sender<Signal>>>>,
}

impl StrategyEngine {
    pub fn subscribe_signals(&self) -> Result<Receiver<Signal>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.signal_subscribers
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?
            .push(sender);
        Ok(receiver)
    }

    pub fn start(
        &self,
        market_data: Receiver<Quote>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<JoinHandle<()>> {
        let strategy = self.strategy.clone();
        let subscribers = self.signal_subscribers.clone();
        let handle = std::thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                match market_data.recv_timeout(Duration::from_millis(100)) {
                    Ok(quote) => {
                        if let Ok(mut strat) = strategy.lock() {
                            if let Some(signal) = strat.process_quote(&quote) {
                                if let Ok(subs) = subscribers.lock() {
                                    for sender in subs.iter() {
                                        let _ = sender.send(signal.clone());
                                    }
                                }
                            }
                        }
                    }

                    Err(_) => {}
                }
            }
        });
        Ok(handle)
    }
}

pub fn new_strategy_engine(strategy: BollingerBandsStrategy) -> StrategyEngine {
    StrategyEngine {
        strategy: Arc::new(Mutex::new(strategy)),
        signal_subscribers: Arc::new(Mutex::new(Vec::new())),
    }
}

pub struct BollingerBandsStrategy {
    price_history: HashMap<String, Vec<f64>>,
    period: usize,
    std_dev_factor: f64,
}

impl BollingerBandsStrategy {
    fn new(period: usize, std_dev_factor: f64) -> Self {
        Self {
            price_history: HashMap::new(),
            period: period,
            std_dev_factor: std_dev_factor,
        }
    }
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
}

pub fn new_bollinger_bands_strategy(period: usize, std_dev_factor: f64) -> BollingerBandsStrategy {
    BollingerBandsStrategy::new(period, std_dev_factor)
}

#[cfg(test)]
#[path = "tests/strategy_engine_test.rs"]
mod strategy_engine_test;
