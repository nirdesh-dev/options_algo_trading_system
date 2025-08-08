use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Quote, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

pub struct StrategyEngine {
    signal_subscribers: Arc<Mutex<Vec<Sender<Signal>>>>,
}

pub struct BollingerBandsStrategy {}

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
        let subscribers = self.signal_subscribers.clone();
        let handle = std::thread::spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                match market_data.recv_timeout(Duration::from_millis(100)) {
                    Ok(quote) => {
                        // Use the proper Signal enum
                        let signal = Signal::Hold {
                            symbol: quote.symbol.clone(),
                        };

                        // Broadcast signal
                        if let Ok(subs) = subscribers.lock() {
                            for sender in subs.iter() {
                                let _ = sender.send(signal.clone());
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

pub fn new_strategy_engine(_strategy: BollingerBandsStrategy) -> StrategyEngine {
    StrategyEngine {
        signal_subscribers: Arc::new(Mutex::new(Vec::new())),
    }
}

pub fn new_bollinger_bands_strategy(
    _period: usize,
    _std_dev_factor: f64,
) -> BollingerBandsStrategy {
    BollingerBandsStrategy {}
}

#[cfg(test)]
#[path = "tests/strategy_engine_test.rs"]
mod strategy_engine_test;
