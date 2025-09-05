use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub last_price: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: u32,
    pub ask_size: u32,
    pub volume: u64,
    pub timestamp: u64,
    pub exchange: Option<String>,
}

#[derive(Debug, Clone, Hash)]
pub struct Subscription {
    pub symbols: Vec<String>,
    pub data_types: Vec<DataType>,
    pub update_frequency: UpdateFrequency,
}

#[derive(Debug, Clone, Hash)]
pub enum DataType {
    RealTimeQuotes,
    HistoricalBars { start: u64, end: u64 },
    Level2,
    Trades,
}

#[derive(Debug, Clone, Hash)]
pub enum UpdateFrequency {
    RealTime,
    Every(std::time::Duration),
    OnDemand,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct MarketDataEvent {
    pub source: String,
    pub source_type: SourceType,
    pub event_type: EventType,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum EventType {
    // From data providers
    Quote {
        provider: String,
        quote: Quote,
    },
    ProviderConnected {
        provider: String,
    },
    ProviderDisconnected {
        provider: String,
    },

    // From service layer
    SubscriptionCreated {
        subscription_id: String,
        symbols: Vec<String>,
        success_count: u32,
        total_providers: u32,
    },
    SubscriptionRemoved {
        subscription_id: String,
        symbols: Vec<String>,
        providers_affected: u32,
    },
    ServiceStarted,
    ServiceStopped,
}

#[derive(Debug, Clone)]
pub enum SourceType {
    DataProvider,
    ServiceLayer,
    Strategy,
}
