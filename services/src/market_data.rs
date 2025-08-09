use anyhow::Result;
use anyhow::anyhow;
use crossbeam_channel::{Receiver, Sender};
use domain::domain::Quote;
use ibapi::prelude::HistoricalBarSize;
use ibapi::prelude::HistoricalWhatToShow;
use ibapi::prelude::RealtimeBarSize;
use ibapi::prelude::RealtimeWhatToShow;
use ibapi::prelude::ToDuration;
use ibapi::{Client, contracts::Contract};
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
    fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>>;

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
    fn init(&self, shutdown: Arc<AtomicBool>) -> Result<JoinHandle<()>> {
        let connection_status = self.connection_status.clone();
        let subscribers = self.subscribers.clone();
        let address = format!("{}:{}", self.host, self.port); // Use struct fields
        let client_id = self.client_id; // Use struct field

        let handle = std::thread::spawn(move || {
            // IBKR Connection
            loop {
                match Client::connect(&address, client_id) {
                    Ok(client) => {
                        connection_status.store(true, std::sync::atomic::Ordering::Relaxed);
                        // Subscribe to market data for symbols
                        let contract = Contract::stock("AAPL");

                        // Comment Real-time pricing to use historical prices for now (since it works outside market hours)

                        // if let Ok(subscription) = client.realtime_bars(
                        //     &contract,
                        //     RealtimeBarSize::Sec5,
                        //     RealtimeWhatToShow::Trades,
                        //     false,
                        // ) {
                        //     for bar in subscription {
                        //         let quote = Quote {
                        //             symbol: "AAPL".to_string(),
                        //             bid: bar.close, // Use close price as both bid/ask for simplicity
                        //             ask: bar.close,
                        //             bid_size: bar.volume as u32,
                        //             ask_size: bar.volume as u32,
                        //             biddate: chrono::Local::now(),
                        //             askdate: chrono::Local::now(),
                        //         };
                        //         if let Ok(subs) = subscribers.lock() {
                        //             for (_, sender) in subs.iter() {
                        //                 let _ = sender.send(quote.clone());
                        //             }
                        //         }

                        //         if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                        //             break;
                        //         }
                        //     }
                        // }

                        // Get historical data instead of real-time
                        match client.historical_data(
                            &contract,
                            None,                   // Current time
                            1.days(),               // Last day of data
                            HistoricalBarSize::Min, // 1-minute bars
                            HistoricalWhatToShow::Trades,
                            true,
                        ) {
                            Ok(historical_data) => {
                                println!(
                                    "✅ Got {} historical bars for streaming",
                                    historical_data.bars.len()
                                );

                                // Stream historical bars as live quotes
                                for bar in historical_data.bars {
                                    if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                                        break;
                                    }

                                    let quote = Quote {
                                        symbol: "AAPL".to_string(),
                                        bid: bar.low,                // Use low as bid
                                        ask: bar.high,               // Use high as ask
                                        bid_size: bar.volume as u32, // Used only as a placeholder to test systems outside market hours
                                        ask_size: bar.volume as u32, // Used only as a placeholder to test systems outside market hours
                                        bid_date: chrono::Local::now(),
                                        ask_date: chrono::Local::now(),
                                    };

                                    if let Ok(subs) = subscribers.lock() {
                                        for (_, sender) in subs.iter() {
                                            let _ = sender.send(quote.clone());
                                        }
                                    }

                                    // Simulate live streaming - send quote every 100ms
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to get historical data: {:?}", e);
                            }
                        }
                        break;
                    }

                    Err(_) => {
                        if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                            break; // Exit if shutting down
                        }
                        std::thread::sleep(std::time::Duration::from_secs(5)); // Retr
                    }
                }
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
