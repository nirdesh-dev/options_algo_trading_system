// mod ib_client;

// use anyhow::Result;
// use ib_client::IBClient;

// use crate::ib_client::IBConfig;

// fn main() -> Result<()> {
//     let ib = IBClient::new(IBConfig::default());
//     let mut client = ib.connect()?;

//     ib.get_spx_minute_data(&mut client)?;

//     Ok(())
// }
use ibapi::prelude::*;

fn main() {
    let connection_url = "127.0.0.1:4002";
    let client = Client::connect(connection_url, 100).expect("connection to TWS failed!");

    // Request real-time bars data for AAPL with 5-second intervals
    let contract = Contract::stock("AAPL");
    let subscription = client
        .realtime_bars(
            &contract,
            RealtimeBarSize::Sec5,
            RealtimeWhatToShow::Trades,
            false,
        )
        .expect("realtime bars request failed!");

    for bar in subscription {
        // Process each bar here (e.g., print or use in calculations)
        println!("bar: {bar:?}");
    }
}
