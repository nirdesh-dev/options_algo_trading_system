use crate::types::{ConnectionStatus, Subscription};
use anyhow::Result;
use async_trait::async_trait;
use domain::Quote;

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    // Connect to the provider
    async fn connect(&self) -> Result<()>;

    // Disconnect from the data provider
    async fn disconnect(&self) -> Result<()>;

    // Subscribe to real-time quotes for symbols
    async fn subscribe(&self, subscription: &Subscription) -> Result<()>;

    // Unsubscribe from symbols
    async fn unsubscribe(&self, symbols: Vec<String>) -> Result<()>;

    // Get connection status
    fn connection_status(&self) -> ConnectionStatus;

    // Get market data provider name
    fn name(&self) -> &str;

    // Health check
    async fn health_check(&self) -> Result<bool>;
}

// Optional trait for providers that also provide historical data
#[async_trait]
pub trait HistoricalDataProvider: MarketDataProvider {
    async fn get_historical_quotes(
        &self,
        symbol: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<Quote>>;
}

// Optional trait for providers with L2 market data
#[async_trait]
pub trait L2DataProvider: MarketDataProvider {
    async fn subscribe_l2(&self, symbols: Vec<String>) -> Result<()>;
}
