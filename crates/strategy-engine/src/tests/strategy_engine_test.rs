use crate::engine::StrategyEngine;
use crate::strategies::bollinger_bands::BollingerBandsStrategy;
use domain::domain::{Quote, Signal};
use market_data::{
    providers::simulator::SimulatorProvider,
    services::MarketDataService,
    types::{DataType, Subscription, UpdateFrequency},
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::signal;

// #[tokio::test]
// async fn test_strategy_engine_basic() {
//     let mut engine = StrategyEngine::new();

//     let strategy = Box::new(BollingerBandsStrategy::new(
//         "test_bb".to_string(),
//         vec!["AAPL".to_string()],
//         3,
//         2.0,
//     ));

//     engine.add_strategy(strategy);

//     // Create test quotes
//     let quotes = vec![
//         Quote {
//             symbol: "AAPL".to_string(),
//             last_price: 150.0,
//             bid: 149.5,
//             ask: 150.5,
//             bid_size: 100,
//             ask_size: 100,
//             volume: 1000,
//             timestamp: 1234567890,
//             exchange: Some("SIMULATOR".to_string()),
//         },
//         Quote {
//             symbol: "AAPL".to_string(),
//             last_price: 151.0,
//             bid: 150.5,
//             ask: 151.5,
//             bid_size: 100,
//             ask_size: 100,
//             volume: 1000,
//             timestamp: 1234567891,
//             exchange: Some("SIMULATOR".to_string()),
//         },
//         Quote {
//             symbol: "AAPL".to_string(),
//             last_price: 152.0,
//             bid: 151.5,
//             ask: 152.5,
//             bid_size: 100,
//             ask_size: 100,
//             volume: 1000,
//             timestamp: 1234567892,
//             exchange: Some("SIMULATOR".to_string()),
//         },
//     ];

//     let mut signal_count = 0;
//     for quote in quotes {
//         engine.process_quote(&quote).unwrap();

//         let signal_receiver = engine.get_signal_receiver();
//         while let Ok(signal) = signal_receiver.try_recv() {
//             eprintln!("Received signal: {:?}", signal);
//             signal_count += 1;
//         }
//     }
//     eprintln!("Total signals generated: {}", signal_count);
//     assert!(
//         signal_count > 0,
//         "Should have generated at least one signal"
//     );
// }

// #[tokio::test]
// async fn test_market_data_to_strategy_integration() {
//     eprintln!("Testing MarketData -> Strategy Engine implementation");

//     let market_data = MarketDataService::new();
//     let provider = SimulatorProvider::new(
//         market_data.get_quote_sender(),
//         market_data.get_event_sender(),
//     );

//     let subscription = Subscription {
//         symbols: vec!["AAPL".to_string()],
//         data_types: vec![DataType::RealTimeQuotes],
//         update_frequency: UpdateFrequency::RealTime,
//     };

//     let sub_id = market_data
//         .subscribe(&provider, subscription)
//         .await
//         .unwrap();
//     eprintln!("Market data subscription creaetd: {:?}", sub_id);

//     let mut strategy_engine = StrategyEngine::new();
//     let strategy = Box::new(BollingerBandsStrategy::new(
//         "bb_aapl".to_string(),
//         vec!["AAPL".to_string()],
//         5,
//         1.5,
//     ));

//     strategy_engine.add_strategy(strategy);
//     eprintln!("Strategy engine created with bollinger bands");

//     let quote_stream = market_data.get_quote_stream();
//     let signal_stream = strategy_engine.get_signal_receiver();

//     let engine_clone = Arc::new(Mutex::new(strategy_engine));
//     let engine_for_thread = engine_clone.clone();

//     let processing_handle = std::thread::spawn(move || {
//         eprintln!("Starting quote processing");
//         // let mut engine = engine_for_thread.lock().unwrap();
//         while let Ok(quote) = quote_stream.recv() {
//             eprintln!(
//                 "Processing quote: {} = ${:.2}",
//                 quote.symbol, quote.last_price
//             );
//             let mut engine = engine_for_thread.lock().unwrap();
//             if let Err(e) = engine.process_quote(&quote) {
//                 eprintln!("Error processing quote: {}", e);
//             }
//         }
//         eprintln!("Quote processing thread stopped");
//     });

//     // // Waiting for background task to generate quotes
//     tokio::time::sleep(Duration::from_millis(1000)).await;
//     eprintln!("Listening for signals");

//     let mut signals_received = 0;
//     let start_time = std::time::Instant::now();

//     while start_time.elapsed() < Duration::from_millis(2000) {
//         match signal_stream.try_recv() {
//             Ok(signal) => {
//                 eprintln!("Signal received: {:?}", signal);
//                 signals_received += 1;
//             }
//             Err(_) => {
//                 tokio::time::sleep(Duration::from_millis(50)).await;
//             }
//         }
//     }
//     eprintln!("Integration test results:");
//     eprintln!("   - Signals received: {}", signals_received);
//     eprintln!("   - Test duration: 2 seconds");

//     // We should have received some signals (at least Hold signals)
//     assert!(
//         signals_received > 0,
//         "Should have received at least one signal from strategy"
//     );
//     eprintln!("Integration test passed!");
// }

#[tokio::test]
async fn test_market_data_to_strategy_integration_with_manual_quote_processing() {
    eprintln!("Testing MarketData -> Strategy Engine implementation");

    let market_data = MarketDataService::new();
    let provider = SimulatorProvider::new(
        market_data.get_quote_sender(),
        market_data.get_event_sender(),
    );

    let subscription = Subscription {
        symbols: vec!["AAPL".to_string()],
        data_types: vec![DataType::RealTimeQuotes],
        update_frequency: UpdateFrequency::RealTime,
    };

    let sub_id = market_data
        .subscribe(&provider, subscription)
        .await
        .unwrap();
    eprintln!("Market data subscription creaetd: {:?}", sub_id);

    let mut strategy_engine = StrategyEngine::new();
    let strategy = Box::new(BollingerBandsStrategy::new(
        "bb_aapl".to_string(),
        vec!["AAPL".to_string()],
        5,
        1.5,
    ));

    strategy_engine.add_strategy(strategy);
    eprintln!("Strategy engine created with bollinger bands");

    let quote_stream = market_data.get_quote_stream();
    let signal_stream = strategy_engine.get_signal_receiver();

    // Waiting for background task to generate quotes
    tokio::time::sleep(Duration::from_millis(1000)).await;
    eprintln!("Listening for signals");

    // Process quotes manually (same as test 1)
    eprintln!("Processing quotes...");
    for i in 0..10 {
        if let Ok(quote) = quote_stream.try_recv() {
            eprintln!("Quote {}: {} = ${:.2}", i, quote.symbol, quote.last_price);
            strategy_engine.process_quote(&quote).unwrap();
        } else {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // Check signals
    eprintln!("Checking for signals...");
    let mut signals_received = 0;
    for _ in 0..5 {
        if let Ok(signal) = signal_stream.try_recv() {
            eprintln!("Signal: {:?}", signal);
            signals_received += 1;
        }
    }

    eprintln!("Signals received: {}", signals_received);
    assert!(signals_received > 0);
}
