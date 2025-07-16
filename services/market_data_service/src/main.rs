mod ib_client;

use anyhow::Result;
use ib_client::IBClient;

use crate::ib_client::IBConfig;

fn main() -> Result<()> {
    let ib = IBClient::new(IBConfig::default());
    let mut client = ib.connect()?;

    ib.get_spx_minute_data(&mut client)?;

    Ok(())
}
