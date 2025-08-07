use super::*;

#[test]
fn test_can_create_ibkr_service() {
    let service = new("127.0.0.1".to_string(), 4002, 100); // Use 127.0.0.1 and client ID 100
}

#[test]
fn test_init_returns_join_handle() {
    let service = new("127.0.0.1".to_string(), 4002, 101);
    let shutdown = Arc::new(AtomicBool::new(false));

    let handle = service.init(shutdown.clone());
    assert!(handle.is_ok());

    // Clean shutdown
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Ok(h) = handle {
        let _ = h.join();
    }
}

#[test]
fn test_service_starts_disconnected() {
    let service = new("127.0.0.1".to_string(), 4002, 102);
    assert!(!service.is_connected());
}

#[test]
fn test_init_attempts_connection() {
    let service = new("127.0.0.1".to_string(), 4002, 103);
    let shutdown = Arc::new(AtomicBool::new(false));

    assert!(!service.is_connected());
    let _handle = service.init(shutdown.clone()).unwrap();

    // Give more time for IBKR connection + historical data fetch
    std::thread::sleep(std::time::Duration::from_millis(3000)); // 3 seconds

    assert!(service.is_connected());
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[test]
fn test_can_subscribe_to_market_data() {
    let service = new("127.0.0.1".to_string(), 4002, 104);
    let symbols = vec!["AAPL".to_string()];

    let receiver = service.subscribe_market_data(symbols).unwrap();
    // Just verify we got a receiver back
}

#[test]
fn test_can_unsubscribe_to_market_data() {
    let service = new("127.0.0.1".to_string(), 4002, 105);
    let symbols = vec!["AAPL".to_string()];

    let receiver = service.subscribe_market_data(symbols).unwrap();

    let result = service.unsubscribe_market_data(&receiver);
    assert!(result.is_ok());

    let result2 = service.unsubscribe_market_data(&receiver);
    assert!(result2.is_err())
}

#[test]
fn test_subscribers_receive_quotes() {
    let service = new("127.0.0.1".to_string(), 4002, 106);

    // Subscribe to market data
    let receiver = service
        .subscribe_market_data(vec!["AAPL".to_string()])
        .unwrap();

    // Start the service
    let shutdown = Arc::new(AtomicBool::new(false));
    let _handle = service.init(shutdown.clone()).unwrap();

    // Wait longer for: connection + historical data fetch + streaming to start
    println!("⏳ Waiting for IBKR connection and historical data...");
    std::thread::sleep(std::time::Duration::from_millis(8000)); // 8 seconds

    // Try to receive a quote with reasonable timeout
    match receiver.recv_timeout(std::time::Duration::from_millis(5000)) {
        // 5 second timeout
        Ok(quote) => {
            println!("✅ Received historical quote: {:?}", quote);
            // Test passes!
        }
        Err(e) => {
            panic!(
                "Expected to receive a quote but got timeout: {:?} - Check IB Gateway is running and connected",
                e
            );
        }
    }

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}
