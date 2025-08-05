use super::*;

#[test]
fn test_can_create_ibkr_service() {
    let service = new("localhost".to_string(), 7497, 1);
}

#[test]
fn test_init_returns_join_handle() {
    let service = new("localhost".to_string(), 7497, 1);
    let shutdown = Arc::new(AtomicBool::new(false));
    let symbols = vec!["AAPL".to_string()];

    let handle = service.init(shutdown.clone(), "localhost".to_string(), 7497, 1);
    assert!(handle.is_ok());

    // Clean shutdown
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Ok(h) = handle {
        let _ = h.join();
    }
}
#[test]
fn test_service_starts_disconnected() {
    let service = new("localhost".to_string(), 7497, 1);
    assert!(!service.is_connected());
}

#[test]
fn test_init_attempts_connection() {
    let service = new("localhost".to_string(), 7497, 1);
    let shutdown = Arc::new(AtomicBool::new(false));

    // Should start disconnected
    assert!(!service.is_connected());
    let _handle = service
        .init(shutdown.clone(), "localhost".to_string(), 7497, 1)
        .unwrap();

    // Give it time to attempt connection
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Should have attempted connection (will fail, but connection_status should be updated)
    // This test will FAIL because our dummy init doesn't try to connect
    assert!(service.is_connected());

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[test]
fn test_can_subscribe_to_market_data() {
    let service = new("localhost".to_string(), 7497, 1);
    let symbols = vec!["AAPL".to_string()];

    let receiver = service.subscribe_market_data(symbols).unwrap();
    // Just verify we got a receiver back
}

#[test]
fn test_can_unsubscribe_to_market_data() {
    let service = new("localhost".to_string(), 7497, 1);
    let symbols = vec!["AAPL".to_string()];

    let receiver = service.subscribe_market_data(symbols).unwrap();

    // This should succeed
    let result = service.unsubscribe_market_data(&receiver);
    assert!(result.is_ok());

    // This should fail - can't unsubscribe twice
    let result2 = service.unsubscribe_market_data(&receiver);
    assert!(result2.is_err())
}

#[test]
fn test_subscribers_receive_quotes() {
    let service = new("localhost".to_string(), 7497, 1);

    // Subscribe to market data
    let receiver = service
        .subscribe_market_data(vec!["AAPL".to_string()])
        .unwrap();

    // Start the service
    let shutdown = Arc::new(AtomicBool::new(false));
    let _handle = service
        .init(shutdown.clone(), "localhost".to_string(), 7497, 1)
        .unwrap();

    // Wait a bit for connection
    std::thread::sleep(std::time::Duration::from_millis(100));

    // TODO: In real implementation, service would receive quotes from IBKR and broadcast them
    // For now, this test will timeout because no quotes are sent

    // Try to receive a quote (this will fail because we don't broadcast any quotes yet)
    match receiver.recv_timeout(std::time::Duration::from_millis(200)) {
        Ok(_quote) => {
            // Test passes if we receive a quote
        }
        Err(_) => {
            panic!("Expected to receive a quote but got timeout");
        }
    }

    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
}
