use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use domain::Quote;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting trading system");

    // Create communication channels
    let (quote_tx, quote_rx) = crossbeam_channel::unbounded::<Quote>();
    let (signal_tx, signal_rx) = crossbeam_channel::unbounded::<String>();

    println!("Channels created");

    // Wait for close signal Ctrl + C
    signal::ctrl_c().await?;
    println!("Shutting down");

    Ok(())
}

fn simulate_market_data(quote_tx: Sender<Quote>) {
    println!("Market data simulator started");

    let mut price = 150.0;

    for i in 0..10 {
        // Send 10 quotes then stop
        price += (i as f64 * 0.5) - 2.0; // Simple price movement

        let quote = Quote {
            symbol: "AAPL".to_string(),
            bid: price - 0.05,
            ask: price + 0.05,
            last_price: price,
            bid_size: 100,
            ask_size: 100,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            volume: 1000,
        };
        println!("Sending quote: AAPL @ ${:.2}", price);
        if quote_tx.send(quote).is_err() {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }
    println!("Market data simulation complete");
}
