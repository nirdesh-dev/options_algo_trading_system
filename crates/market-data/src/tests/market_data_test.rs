use crate::providers::simulator::SimulatorProvider;
use crate::services::MarketDataService;
use crate::traits::MarketDataProvider;
use crate::types::{ConnectionStatus, DataType, Quote, Subscription, UpdateFrequency};
use crossbeam_channel::unbounded;
use tokio::time::Duration;

#[tokio::test]
async fn test_simulator_connection_status() {
    let (quote_tx, _quote_rx) = crossbeam_channel::unbounded();
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    let provider = SimulatorProvider::new(quote_tx, event_tx);

    // Should start disconnected
    println!("Initial status: {:?}", provider.connection_status());
    assert_eq!(provider.connection_status(), ConnectionStatus::Disconnected);

    // Should connect
    provider.connect().await.unwrap();
    println!("After connect: {:?}", provider.connection_status());
    assert_eq!(provider.connection_status(), ConnectionStatus::Connected);

    // Should disconnect
    provider.disconnect().await.unwrap();
    println!("After disconnect: {:?}", provider.connection_status());
    assert_eq!(provider.connection_status(), ConnectionStatus::Disconnected);
}

#[tokio::test]
async fn test_quote_channel_basic() {
    let (quote_tx, quote_rx) = crossbeam_channel::unbounded();

    let test_quote = Quote {
        symbol: "TEST".to_string(),
        last_price: 100.0,
        bid: 99.5,
        ask: 100.5,
        bid_size: 100,
        ask_size: 200,
        volume: 1000,
        timestamp: 1234567890,
        exchange: Some("TEST_EXCHANGE".to_string()),
    };

    println!("Sending test quote...");

    quote_tx.send(test_quote.clone()).unwrap();

    println!("Receiving test quote...");

    let received = quote_rx.recv().unwrap();

    assert_eq!(received.symbol, test_quote.symbol);
    assert_eq!(received.last_price, test_quote.last_price);

    println!("✅ Channel test passed");
}

#[tokio::test]
async fn test_simulator_stores_subscription() {
    let (quote_tx, _quote_rx) = crossbeam_channel::unbounded();
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    let provider = SimulatorProvider::new(quote_tx, event_tx);

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    provider.subscribe(&subscription).await.unwrap();

    // Verify subscription was stored
    let subscriptions = provider.get_subscriptions().await;
    assert!(subscriptions.contains_key("AAPL"));
    println!("✅ Subscription stored correctly");
}

#[tokio::test]
async fn test_simulator_rejects_unsupported_data_types() {
    let (quote_tx, _quote_rx) = crossbeam_channel::unbounded();
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    let provider = SimulatorProvider::new(quote_tx, event_tx);

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string()],
        data_types: vec![DataType::Level2], // Unsupported!
        update_frequency: UpdateFrequency::RealTime,
    };

    let result = provider.subscribe(&subscription).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Level2 data not supported"));
    println!("✅ Correctly rejects unsupported data types");
}

#[tokio::test]
async fn test_simulator_initializes_base_prices() {
    let (quote_tx, _quote_rx) = crossbeam_channel::unbounded();
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    let provider = SimulatorProvider::new(quote_tx, event_tx);

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string(), "MSFT".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    provider.subscribe(&subscription).await.unwrap();

    // Give tasks time to initialize prices
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Check if base prices were set (they will have changed due to price movement)
    let prices = provider.get_base_prices().await;
    assert!(prices.contains_key("AAPL"));
    assert!(prices.contains_key("MSFT"));

    // Just verify prices are reasonable (not exact values)
    let aapl_price = prices.get("AAPL").unwrap();
    let msft_price = prices.get("MSFT").unwrap();

    assert!(*aapl_price > 140.0 && *aapl_price < 160.0); // Around 150.0
    assert!(*msft_price > 240.0 && *msft_price < 260.0); // Around 250.0

    println!(
        "✅ Base prices initialized correctly: AAPL=${:.2}, MSFT=${:.2}",
        aapl_price, msft_price
    );
}

#[tokio::test]
async fn test_simulator_generates_quotes_directly() {
    let (quote_tx, quote_rx) = crossbeam_channel::unbounded();
    let (event_tx, _event_rx) = crossbeam_channel::unbounded();

    let provider = SimulatorProvider::new(quote_tx, event_tx);

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    provider.subscribe(&subscription).await.unwrap();
    println!("Subscribe completed, waiting for background tasks");

    tokio::time::sleep(Duration::from_millis(300)).await;
    println!("Checking for quotes");

    match quote_rx.try_recv() {
        Ok(quote) => {
            println!(
                "Received quote: symbol = {}, price = {}",
                quote.symbol, quote.last_price
            );
            assert_eq!(quote.symbol, "AAPL");
        }
        Err(e) => {
            println!("No quotes available: {:?}", e);
            panic!("Should receive quotes after 300 ms")
        }
    }
}

#[tokio::test]
async fn test_market_data_service_channels() {
    let service = MarketDataService::new();

    let quote_sender = service.get_quote_sender();
    let quote_receiver = service.get_quote_stream();

    let test_quote = Quote {
        symbol: "TEST".to_string(),
        last_price: 100.0,
        bid: 99.5,
        ask: 100.5,
        bid_size: 100,
        ask_size: 200,
        volume: 1000,
        timestamp: 1234567890,
        exchange: Some("TEST".to_string()),
    };

    quote_sender.send(test_quote.clone()).unwrap();

    // Receive from service
    let received = quote_receiver.recv().unwrap();
    assert_eq!(received.symbol, test_quote.symbol);
}

#[tokio::test]
async fn test_service_provider_integration() {
    let service = MarketDataService::new();

    let provider = SimulatorProvider::new(service.get_quote_sender(), service.get_event_sender());

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    let sub_id = service.subscribe(&provider, subscription).await.unwrap();
    println!("Subscription created: {}", sub_id);

    // Wait for background tasks
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Try and get quotes from service
    let quote_receiver = service.get_quote_stream();

    match quote_receiver.try_recv() {
        Ok(quote) => {
            println!(
                "Integration works: symbol={}, price={}",
                quote.symbol, quote.last_price
            );
            assert_eq!(quote.symbol, "AAPL");
        }
        Err(e) => {
            println!("Integration failed: {:?}", e);
            panic!("Should receive quotes through service integration");
        }
    }
}
