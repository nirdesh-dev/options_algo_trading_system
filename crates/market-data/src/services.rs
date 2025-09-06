use crate::traits::MarketDataProvider;
use crate::types::{ConnectionStatus, EventType, MarketDataEvent, SourceType, Subscription};
use anyhow::Result;
use crossbeam_channel::{unbounded, Receiver, Sender};
use domain::Quote;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

pub struct MarketDataService {
    subscriptions: Arc<RwLock<HashMap<String, (String, Subscription)>>>,
    quote_sender: Sender<Quote>,
    quote_receiver: Receiver<Quote>,
    event_sender: Sender<MarketDataEvent>,
    event_receiver: Receiver<MarketDataEvent>,
}

impl MarketDataService {
    pub fn new() -> Self {
        let (quote_sender, quote_receiver) = unbounded();
        let (event_sender, event_receiver) = unbounded();

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            quote_sender,
            quote_receiver,
            event_sender,
            event_receiver,
        }
    }

    fn generate_subscription_id(&self, subscription: &Subscription) -> String {
        let mut hasher = DefaultHasher::new();
        subscription.hash(&mut hasher);
        format!("sub_{:x}", hasher.finish())
    }

    // Provider-specific subscription
    pub async fn subscribe<P: MarketDataProvider>(
        &self,
        provider: &P,
        subscription: Subscription,
    ) -> Result<String> {
        let subscription_id = self.generate_subscription_id(&subscription);

        // Connect provider if needed
        if provider.connection_status() != ConnectionStatus::Connected {
            provider.connect().await?;
        }

        // Subscribe directly to provider
        provider.subscribe(&subscription).await?;

        // Store subscription with provider name
        self.subscriptions.write().await.insert(
            subscription_id.clone(),
            (provider.name().to_string(), subscription.clone()),
        );

        // Emit event
        let event = MarketDataEvent {
            source: provider.name().to_string(),
            source_type: SourceType::DataProvider,
            event_type: EventType::SubscriptionCreated {
                subscription_id: subscription_id.clone(),
                symbols: subscription.symbols,
                success_count: 1,
                total_providers: 1,
            },
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        };
        let _ = self.event_sender.send(event);

        Ok(subscription_id)
    }

    // Provider-specific unsubscribe
    pub async fn unsubscribe<P: MarketDataProvider>(
        &self,
        provider: &P,
        subscription_id: &str,
    ) -> Result<()> {
        if let Some((stored_provider_name, subscription)) =
            self.subscriptions.write().await.remove(subscription_id)
        {
            // Verify provider matches
            if stored_provider_name != provider.name() {
                return Err(anyhow::anyhow!(
                    "Provider mismatch: subscription belongs to {}, not {}",
                    stored_provider_name,
                    provider.name()
                ));
            }

            provider.unsubscribe(subscription.symbols.clone()).await?;

            // Emit event
            let event = MarketDataEvent {
                source: provider.name().to_string(),
                source_type: SourceType::DataProvider,
                event_type: EventType::SubscriptionRemoved {
                    subscription_id: subscription_id.to_string(),
                    symbols: subscription.symbols,
                    providers_affected: 1,
                },
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            };
            let _ = self.event_sender.send(event);

            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Subscription {} not found",
                subscription_id
            ))
        }
    }

    pub fn get_quote_stream(&self) -> Receiver<Quote> {
        self.quote_receiver.clone()
    }

    pub fn get_event_stream(&self) -> Receiver<MarketDataEvent> {
        self.event_receiver.clone()
    }

    pub fn get_quote_sender(&self) -> Sender<Quote> {
        self.quote_sender.clone()
    }

    pub fn get_event_sender(&self) -> Sender<MarketDataEvent> {
        self.event_sender.clone()
    }
}
