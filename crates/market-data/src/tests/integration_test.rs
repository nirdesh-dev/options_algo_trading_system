use crate::providers::simulator::SimulatorProvider;
use crate::services::MarketDataService;
use crate::traits::MarketDataProvider;
use crate::types::{DataType, Subscription, UpdateFrequency};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_market_data_service_ingestion() {
    // Create service
    let service = MarketDataService::new();

    // // Create provider with service's channels
    let provider = SimulatorProvider::new(service.get_quote_sender(), service.get_event_sender());

    println!(
        "Initial provider status: {:?}",
        provider.connection_status()
    );

    // // Create subscription
    let subscription = Subscription {
        symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    // // Subscribe
    let sub_id = service.subscribe(&provider, subscription).await.unwrap();
    println!("Created subscription: {}", sub_id);

    // // Listen for quotes for a short time
    let quote_stream = service.get_quote_stream();
    let event_stream = service.get_event_stream();

    // Wait for background tasks
    tokio::time::sleep(Duration::from_millis(300)).await;

    // // Test quote reception
    let quote_result = timeout(Duration::from_secs(2), async { quote_stream.recv() })
        .await
        .unwrap();

    assert!(
        quote_result.is_ok(),
        "Should receive quotes within 2 seconds"
    );
    let quote = quote_result.unwrap();
    println!("Received quote: {:?}", quote);

    // Test event reception
    let event_result = timeout(Duration::from_secs(1), async { event_stream.recv() }).await;

    assert!(event_result.is_ok(), "Should receive events");
    let event = event_result.unwrap().unwrap();
    println!("Received event: {:?}", event);

    // Unsubscribe
    service.unsubscribe(&provider, &sub_id).await.unwrap();
    println!("Unsubscribed successfully");
}
