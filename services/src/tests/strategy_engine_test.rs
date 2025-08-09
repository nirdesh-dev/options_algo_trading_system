use crossbeam::epoch::Atomic;
use ibapi::market_data;

use super::*;
use domain::domain::Quote;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[test]
fn test_can_create_strategy_engine() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);
}

#[test]
fn test_strategy_engine_has_subscribe_method() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);

    let _receiver = engine.subscribe_signals();
}

#[test]
fn test_strategy_engine_can_start() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);

    // Create a mock market data channel
    let (_sender, market_data_receiver) = crossbeam_channel::unbounded();
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = engine.start(market_data_receiver, shutdown);
    assert!(handle.is_ok());
}

#[test]
fn test_strategy_engine_processes_quotes_and_generates_signals() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);

    // Subscribe to signals
    let signal_receiver = engine.subscribe_signals().unwrap();

    // Create market data channel and send a quote
    let (market_sender, market_receiver) = crossbeam_channel::unbounded();
    let shutdown = Arc::new(AtomicBool::new(false));

    // Start the engine
    let _handle = engine.start(market_receiver, shutdown.clone()).unwrap();

    // Send a mock quote
    let mock_quote = Quote {
        symbol: "AAPL".to_string(),
        bid: 100.0,
        ask: 101.0,
        bid_size: 100,
        ask_size: 100,
        bid_date: chrono::Local::now(),
        ask_date: chrono::Local::now(),
    };

    market_sender.send(mock_quote).unwrap();

    // Wait a bit for processing
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Try to receive a signal - this will fail because our engine doesn't process quotes yet
    match signal_receiver.recv_timeout(std::time::Duration::from_millis(200)) {
        Ok(_signal) => {
            // Test passes if we receive any signal
        }
        Err(_) => {
            panic!("Expected to receive a signal but got timeout");
        }
    }
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[test]
fn test_bollinger_bands_generates_buy_signal_at_lower_band() {
    let strategy = new_bollinger_bands_strategy(3, 2.0);
    let engine = new_strategy_engine(strategy);

    let signal_receiver = engine.subscribe_signals().unwrap();
    let (market_sender, market_receiver) = crossbeam_channel::unbounded();
    let shutdown = Arc::new(AtomicBool::new(false));

    let _handle = engine.start(market_receiver, shutdown.clone()).unwrap();
    // Send quotes to establish a pattern: high prices first, then a low price
    let quotes = vec![
        Quote {
            symbol: "AAPL".to_string(),
            bid: 199.0,
            ask: 201.0, // High price
            bid_size: 100,
            ask_size: 100,
            bid_date: chrono::Local::now(),
            ask_date: chrono::Local::now(),
        },
        Quote {
            symbol: "AAPL".to_string(),
            bid: 198.0,
            ask: 202.0, // High price
            bid_size: 100,
            ask_size: 100,
            bid_date: chrono::Local::now(),
            ask_date: chrono::Local::now(),
        },
        Quote {
            symbol: "AAPL".to_string(),
            bid: 99.0,
            ask: 101.0, // Very low price - should trigger BUY signal
            bid_size: 100,
            ask_size: 100,
            bid_date: chrono::Local::now(),
            ask_date: chrono::Local::now(),
        },
    ];

    for quote in quotes {
        market_sender.send(quote).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    match signal_receiver.recv_timeout(std::time::Duration::from_millis(500)) {
        Ok(signal) => {
            match signal {
                Signal::Buy {
                    symbol,
                    price,
                    confidence: _,
                } => {
                    assert_eq!(symbol, "AAPL");
                    assert!((price - 100.0).abs() < 1.0); // Price should be around 100
                }
                _ => panic!("Expected BUY signal, got {:?}", signal),
            }
        }
        Err(_) => {
            panic!("Expected to receive a BUY signal but got timeout");
        }
    }

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}
