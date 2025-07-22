mod config;
mod data_loader;
mod domain;
mod strategy;

use crate::config::DomainRules;
use crate::data_loader::DataLoader;
use crate::domain::pricing::PricingModel;
use crate::strategy::iron_condor::config::{IronCondorConfig, IronCondorRawConfig};
use crate::strategy::iron_condor::iron_condor::IronCondor;
use anyhow::Result;
use polars::prelude::*;
use std::fs::File;

fn main() -> Result<()> {
    // Hardcoded domain rules (normally from config.toml)
    // let domain_rules = DomainRules {
    //     entry_time_min: 15,
    //     entry_time_max: 360,
    //     wing_width_min: 1.0,
    //     wing_width_max: 100.0,
    //     stop_loss_min: 0.5,
    //     stop_loss_max: 5.0,
    // };

    // // Hardcoded raw iron condor config (normally from config.toml)
    // let raw_iron_condor_config = IronCondorRawConfig {
    //     entry_times: vec![15, 60, 120, 180],
    //     wing_widths: vec![5.0, 10.0, 15.0],
    //     stop_losses: vec![1.0, 2.0, 3.0],
    // };

    // // Convert raw config into domain-typed validated config
    // let iron_condor_config = IronCondorConfig::from_raw(raw_iron_condor_config, &domain_rules)?;

    // // Create the IronCondor strategy instance
    // let strategy = IronCondor::new(iron_condor_config);

    // // Dummy price series for testing (1000 points of price 100.0)
    // let price_series = vec![100.0_f32; 1000];

    // // Run the GPU backtest with Black-Scholes pricing model
    // let results = strategy.run_backtest_on_gpu(&price_series, PricingModel::BlackScholes)?;

    // // Print the results summary
    // for (idx, result) in results.iter().enumerate() {
    //     println!(
    //         "Params set {}: PnL = {:.2}, Drawdown = {:.2}, Sharpe = {:.2}",
    //         idx, result.total_pnl, result.drawdown, result.sharpe,
    //     );
    // }

    let opt_data_loader = DataLoader::from_yaml("data_config.yaml")?;

    let mut backtest_data = opt_data_loader.load_all_data()?;

    println!("{:}", backtest_data.joined_data);

    let mut file = File::create("spx_options.csv")?;
    CsvWriter::new(&mut file).finish(&mut backtest_data.joined_data)?;

    // println!("{:?}", backtest_data.describe());

    Ok(())
}
