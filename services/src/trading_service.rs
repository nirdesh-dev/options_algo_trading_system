use anyhow::Result;
use crossbeam::epoch::Atomic;
use crossbeam_channel::Receiver;
use domain::domain::Signal;
use ibapi::Client;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub side: String, // Buy or Sell
    pub quantity: u32,
    pub price: f64,
}

pub struct TradingService {
    host: String,
    port: u16,
    client_id: i32,
    orders: Arc<Mutex<Vec<Order>>>,
    ibkr_connected: Arc<AtomicBool>,
}

impl TradingService {
    pub fn start(
        &self,
        shutdown: Arc<AtomicBool>,
        signal_receiver: Receiver<Signal>,
    ) -> Result<JoinHandle<()>> {
        let orders = self.orders.clone();

        let ibkr_connected = self.ibkr_connected.clone();
        let address = format!("{}:{}", self.host, self.port);
        let client_id = self.client_id;

        let handle = std::thread::spawn(move || {
            let _client = match Client::connect(&address, client_id) {
                Ok(client) => {
                    ibkr_connected.store(true, Ordering::Relaxed);
                    println!("Connecting to IBKR for trading");
                    Some(client) // Keep client alive
                }
                Err(e) => {
                    println!("Failed to connect to IBKR: {}", e);
                    None
                }
            };
            while !shutdown.load(Ordering::Relaxed) {
                match signal_receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(signal) => {
                        match signal {
                            Signal::Buy {
                                symbol,
                                price,
                                confidence: _,
                            } => {
                                // Create and "place" a buy order
                                let order = Order {
                                    symbol,
                                    side: "BUY".to_string(),
                                    quantity: 100, // Default quantity
                                    price,
                                };
                                if let Ok(mut order_list) = orders.lock() {
                                    order_list.push(order);
                                    println!("Placed Buy order");
                                }
                            }
                            Signal::Sell {
                                symbol,
                                price,
                                confidence: _,
                            } => {
                                // Create and "place" a sell order
                                let order = Order {
                                    symbol,
                                    side: "SELL".to_string(),
                                    quantity: 100,
                                    price,
                                };

                                if let Ok(mut order_list) = orders.lock() {
                                    order_list.push(order);
                                    println!("Placed SELL order");
                                }
                            }
                            Signal::Hold { symbol } => {}
                        }
                    }
                    Err(_) => {}
                }
            }
        });
        Ok(handle)
    }

    pub fn get_orders(&self) -> Result<Vec<Order>> {
        self.orders
            .lock()
            .map(|orders| orders.clone())
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))
    }

    pub fn is_connected_to_ibkr(&self) -> bool {
        self.ibkr_connected.load(Ordering::Relaxed)
    }
}

pub fn new_trading_service(host: String, port: u16, client_id: i32) -> TradingService {
    TradingService {
        host,
        port,
        client_id,
        orders: Arc::new(Mutex::new(Vec::new())),
        ibkr_connected: Arc::new(AtomicBool::new(false)),
    }
}

#[cfg(test)]
#[path = "tests/trading_service_test.rs"]
mod trading_service_test;
