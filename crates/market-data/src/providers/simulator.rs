use crate::traits::MarketDataProvider;
use crate::types::{
    ConnectionStatus, DataType, EventType, MarketDataEvent, SourceType, Subscription,
    UpdateFrequency,
};
use anyhow::Result;
use async_trait::async_trait;
use crossbeam_channel::Sender;
use domain::Quote;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct SimulatorProvider {
    name: String,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    quote_sender: Option<Sender<Quote>>,
    event_sender: Option<Sender<MarketDataEvent>>,
    base_prices: Arc<RwLock<HashMap<String, f64>>>,
}

impl SimulatorProvider {
    pub fn new(quote_sender: Sender<Quote>, event_sender: Sender<MarketDataEvent>) -> Self {
        Self {
            name: "SimulatorProvider".to_string(),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            quote_sender: Some(quote_sender),
            event_sender: Some(event_sender),
            base_prices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn start_quote_generation(&self, symbol: String) {
        let quote_sender = self.quote_sender.clone();
        let base_prices = self.base_prices.clone();
        let provider_name = self.name.clone();

        tokio::spawn(async move {
            if let Some(sender) = quote_sender {
                let mut interval = interval(Duration::from_millis(100)); // 10 quotes per second

                // Initialize base price if not exists
                {
                    let mut prices = base_prices.write().await;
                    if !prices.contains_key(&symbol) {
                        let base_price = match symbol.as_str() {
                            "AAPL" => 150.0,
                            "MSFT" => 250.0,
                            "GOOGL" => 2500.0,
                            "TSLA" => 200.0,
                            _ => 100.0,
                        };
                        prices.insert(symbol.clone(), base_price);
                    }
                }

                loop {
                    interval.tick().await;

                    // Generate realistic price movement
                    let price = {
                        let mut prices = base_prices.write().await;
                        if let Some(current_price) = prices.get_mut(&symbol) {
                            // Random walk: ±0.1% change
                            let change_percent = (rand::random::<f64>() - 0.5) * 0.002;
                            *current_price *= 1.0 + change_percent;
                            *current_price
                        } else {
                            continue;
                        }
                    };

                    let quote = Quote {
                        symbol: symbol.clone(),
                        last_price: price,
                        bid: price - 0.01,
                        ask: price + 0.01,
                        bid_size: rand::random::<u32>() % 1000 + 100,
                        ask_size: rand::random::<u32>() % 1000 + 100,
                        volume: rand::random::<u64>() % 10000 + 1000,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        exchange: Some("SIMULATOR".to_string()),
                    };

                    if sender.send(quote).is_err() {
                        break; // Receiver dropped
                    }
                }
            }
        });
    }

    #[cfg(test)]
    pub async fn get_subscriptions(&self) -> HashMap<String, Subscription> {
        self.subscriptions.read().await.clone()
    }

    #[cfg(test)]
    pub async fn get_base_prices(&self) -> HashMap<String, f64> {
        self.base_prices.read().await.clone()
    }
}

#[async_trait]
impl MarketDataProvider for SimulatorProvider {
    async fn connect(&self) -> Result<()> {
        *self.connection_status.write().await = ConnectionStatus::Connecting;

        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;

        *self.connection_status.write().await = ConnectionStatus::Connected;

        // Emit connection event
        if let Some(sender) = &self.event_sender {
            let event = MarketDataEvent {
                source: self.name.clone(),
                source_type: SourceType::DataProvider,
                event_type: EventType::ProviderConnected {
                    provider: self.name.clone(),
                },
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            let _ = sender.send(event);
        }

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        *self.connection_status.write().await = ConnectionStatus::Disconnected;
        self.subscriptions.write().await.clear();

        // Emit disconnection event
        if let Some(sender) = &self.event_sender {
            let event = MarketDataEvent {
                source: self.name.clone(),
                source_type: SourceType::DataProvider,
                event_type: EventType::ProviderDisconnected {
                    provider: self.name.clone(),
                },
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            let _ = sender.send(event);
        }

        Ok(())
    }

    async fn subscribe(&self, subscription: &Subscription) -> Result<()> {
        // Check if we support the requested data types
        for data_type in &subscription.data_types {
            match data_type {
                DataType::RealTimeQuotes => { /* supported */ }
                DataType::HistoricalBars { .. } => {
                    return Err(anyhow::anyhow!(
                        "Historical data not supported by SimulatorProvider"
                    ));
                }
                DataType::Level2 => {
                    return Err(anyhow::anyhow!(
                        "Level2 data not supported by SimulatorProvider"
                    ));
                }
                DataType::Trades => {
                    return Err(anyhow::anyhow!(
                        "Trade data not supported by SimulatorProvider"
                    ));
                }
            }
        }
        println!("Requested format data is supported");

        // Store subscription
        for symbol in &subscription.symbols {
            self.subscriptions
                .write()
                .await
                .insert(symbol.clone(), subscription.clone());

            // Start generating quotes for this symbol
            self.start_quote_generation(symbol.clone()).await;
        }

        Ok(())
    }

    async fn unsubscribe(&self, symbols: Vec<String>) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        for symbol in symbols {
            subscriptions.remove(&symbol);
        }
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        match self.connection_status.try_read() {
            Ok(status) => status.clone(),
            Err(_) => ConnectionStatus::Disconnected, // Fallback if lock is held
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(matches!(
            *self.connection_status.read().await,
            ConnectionStatus::Connected
        ))
    }
}
