// use anyhow::{Context, Result};
// use ibapi::prelude::*;
// use time::macros::datetime;

// #[derive(Debug, Clone)]
// pub struct IBConfig {
//     pub host: String,
//     pub port: u16,
//     pub client_id: i32,
// }

// impl Default for IBConfig {
//     fn default() -> Self {
//         Self {
//             host: "127.0.0.1".to_string(),
//             port: 4002,
//             client_id: 100,
//         }
//     }
// }

// pub struct IBClient {
//     config: IBConfig,
// }

// impl IBClient {
//     pub fn new(config: IBConfig) -> Self {
//         Self { config }
//     }

//     pub fn connect(&self) -> Result<Client> {
//         let connection_url = format!("{}:{}", self.config.host, self.config.port);
//         let client =
//             Client::connect(&connection_url, self.config.client_id).with_context(|| {
//                 format!("Failed to connect to IB Gateway/TWS at {}", connection_url)
//             })?;
//         Ok(client)
//     }

//     pub fn get_spx_minute_data(&self, client: &mut Client) -> Result<()> {
//         let spx_contract = Contract {
//             symbol: "SPX".to_string(),
//             security_type: SecurityType::Index,
//             exchange: "CBOE".to_string(),
//             ..Default::default()
//         };

//         let now = datetime!(2025-07-15 20:00 UTC); // replace with `time::OffsetDateTime::now_utc()` if using time crate fully

//         let historical = client
//             .historical_data(
//                 &spx_contract,
//                 Some(now),
//                 1.days(),
//                 HistoricalBarSize::Min,
//                 HistoricalWhatToShow::Trades,
//                 true,
//             )
//             .context("Failed to get historical data")?;

//         println!("SPX 1-min bars ({} total):", historical.bars.len());
//         for bar in &historical.bars {
//             println!(
//                 "{} | Open: {:.2}, High: {:.2}, Low: {:.2}, Close: {:.2}, Volume: {}",
//                 bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume
//             );
//         }

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_ib_client_can_be_created_with_default_config() {
//         // Arrange
//         let config = IBConfig::default();

//         // Act
//         let client = IBClient::new(config);

//         // Assert
//         assert_eq!(client.config.host, "127.0.0.1");
//         assert_eq!(client.config.port, 4002);
//         assert_eq!(client.config.client_id, 100);
//     }

//     #[test]
//     fn test_ib_client_can_be_created_with_custom_config() {
//         // Arrange
//         let config = IBConfig {
//             host: "192.168.1.100".to_string(),
//             port: 7497,
//             client_id: 200,
//         };

//         // Act
//         let client = IBClient::new(config.clone()); // Clone to use in asserts

//         // Assert
//         assert_eq!(client.config.host, config.host);
//         assert_eq!(client.config.port, config.port);
//         assert_eq!(client.config.client_id, config.client_id);
//     }
// }
