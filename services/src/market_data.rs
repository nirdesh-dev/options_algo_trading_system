use anyhow::Result;
use anyhow::anyhow;
use crossbeam::epoch::Atomic;
use crossbeam_channel::{Receiver, Sender};
use domain::domain::Quote;
use std::any;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct MarketData {
    pub host: String,
    pub port: u16,
    pub client_id: i32,
    pub subscribers: Arc<Mutex<Vec<(u32, Sender<Quote>)>>>, // Changed: use ID instead of storing receiver
    pub connection_status: Arc<AtomicBool>,
    pub request_counter: Arc<AtomicI32>,
    pub subscriber_counter: Arc<AtomicU32>, // Add this for unique subscriber IDs
}

pub trait MarketDataService {
    /// Initialize connection to TWS/IB Gateway
    fn init(
        &self,
        shutdown: Arc<AtomicBool>,
        host: String,
        port: u16,
        client_id: i32,
    ) -> Result<JoinHandle<()>>;

    /// Subscribe to real-time market data for symbols
    fn subscribe_market_data(&self, symbols: Vec<String>) -> Result<Receiver<Quote>>;

    /// Unsubscribe from market data
    fn unsubscribe_market_data(&self, subscriber: &Receiver<Quote>) -> Result<()>;

    /// Check if connected to TWS/Gateway
    fn is_connected(&self) -> bool; // Add this method
}

pub fn new(host: String, port: u16, client_id: i32) -> Arc<impl MarketDataService> {
    Arc::new(MarketData {
        host,
        port,
        client_id,
        subscribers: Arc::new(Mutex::new(Vec::new())),
        connection_status: Arc::new(AtomicBool::new(false)),
        request_counter: Arc::new(AtomicI32::new(1)),
        subscriber_counter: Arc::new(AtomicU32::new(1)),
    })
}

impl MarketDataService for MarketData {
    fn init(
        &self,
        shutdown: Arc<AtomicBool>,
        _host: String,
        _port: u16,
        _client_id: i32,
    ) -> Result<JoinHandle<()>> {
        let connection_status = self.connection_status.clone();
        let subscribers = self.subscribers.clone();
        let handle = std::thread::spawn(move || {
            // Simulate connection attempt
            // For now, just set connection status to true
            // Later we'll replace this with real IBKR connection logic
            connection_status.store(true, std::sync::atomic::Ordering::Relaxed);

            while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                // Main loop - sending mock quotes
                let mock_quote = Quote {
                    symbol: "AAPL".to_string(),
                    bid: 150.0,
                    ask: 150.5,
                    bid_size: 100,
                    ask_size: 100,
                    biddate: chrono::Local::now(),
                    askdate: chrono::Local::now(),
                };

                // Broadcast to all subscribers
                if let Ok(subs) = subscribers.lock() {
                    for (_, sender) in subs.iter() {
                        let _ = sender.send(mock_quote.clone());
                    }
                }

                // Sleep INSIDE the loop to prevent busy waiting
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Set disconnected status AFTER loop exits
            connection_status.store(false, std::sync::atomic::Ordering::Relaxed);
        });
        Ok(handle)
    }

    fn subscribe_market_data(&self, _symbols: Vec<String>) -> Result<Receiver<Quote>> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let subscriber_id = self
            .subscriber_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.subscribers
            .lock()
            .map_err(|e| anyhow!("Lock error: {}", e))?
            .push((subscriber_id, sender));
        Ok(receiver)
    }

    fn unsubscribe_market_data(&self, subscriber: &Receiver<Quote>) -> Result<()> {
        let mut subscribers = self
            .subscribers
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        // Find and remove the subscriber
        if !subscribers.is_empty() {
            subscribers.remove(0); // Remove first subscriber for now
            Ok(())
        } else {
            Err(anyhow::anyhow!("No subscribers to remove"))
        }
    }

    fn is_connected(&self) -> bool {
        self.connection_status
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
#[path = "tests/market_data_test.rs"]
mod market_data_test;
