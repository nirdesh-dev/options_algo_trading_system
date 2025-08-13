use super::*;
use crossbeam::epoch::Atomic;
use domain::domain::Signal;
use ibapi::orders::Order;
use std::net::Shutdown;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

// #[test]
// fn test_can_create_trading_service() {
//     let service = new_trading_service("127.0.0.1".to_string(), 4002, 300);
// }

// #[test]
// fn test_trading_service_can_start() {
//     let service = new_trading_service("127.0.0.1".to_string(), 4002, 300);
//     let shutdown = Arc::new(AtomicBool::new(false));

//     let (_sender, signal_receiver) = crossbeam_channel::unbounded();

//     let handle = service.start(shutdown, signal_receiver);
//     assert!(handle.is_ok());
// }

// #[test]
// fn test_trading_service_processes_buy_signals() {
//     let service = new_trading_service("127.0.0.1".to_string(), 4002, 300);

//     // Create signal channel
//     let (signal_sender, signal_receiver) = crossbeam_channel::unbounded();
//     let shutdown = Arc::new(AtomicBool::new(false));

//     let handle = service.start(shutdown.clone(), signal_receiver);
//     assert!(handle.is_ok());

//     // Send a buy signal
//     let buy_signal = Signal::Buy {
//         symbol: "AAPL".to_string(),
//         price: 150.0,
//         confidence: 0.8,
//     };
//     signal_sender.send(buy_signal).unwrap();

//     // Small delay: Wait for processing
//     std::thread::sleep(Duration::from_millis(100));

//     shutdown.store(true, Ordering::Relaxed);
// }

#[test]
fn test_trading_service_places_buy_order() {
    let service = new_trading_service("127.0.0.1".to_string(), 4002, 500);

    let (signal_sender, signal_receiver) = crossbeam_channel::unbounded();
    let shutdown = Arc::new(AtomicBool::new(false));
    let handle = service.start(shutdown.clone(), signal_receiver).unwrap();

    // Send a BUY signal
    let buy_signal = Signal::Buy {
        symbol: "AAPL".to_string(),
        price: 150.0,
        confidence: 0.8,
    };
    signal_sender.send(buy_signal).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(200));

    println!("{:?}", service.get_orders().unwrap());

    assert!(service.get_orders().unwrap().len() > 0);

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    let _ = handle.join(); // Wait for thread to actually finish
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[test]
fn test_trading_service_places_real_ibkr_order() {
    let service = new_trading_service("127.0.0.1".to_string(), 4002, 400);

    let (signal_sender, signal_receiver) = crossbeam_channel::unbounded();
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = service.start(shutdown.clone(), signal_receiver).unwrap();

    // Send a BUY signal
    let buy_signal = Signal::Buy {
        symbol: "AAPL".to_string(),
        price: 150.0,
        confidence: 0.8,
    };
    signal_sender.send(buy_signal).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(500));

    assert!(service.is_connected_to_ibkr());

    shutdown.store(true, Ordering::Relaxed);

    let _ = handle.join(); // Wait for thread to actually finish
    std::thread::sleep(std::time::Duration::from_millis(100));
}
