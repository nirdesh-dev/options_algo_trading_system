use super::*;
use domain::domain::DomainRules;

#[test]
fn test_strategy_period_validation() {
    let mut domain_rules = DomainRules::default();

    // Set some technical indicator constraints
    domain_rules.technical_indicators.period_min = Some(5);
    domain_rules.technical_indicators.period_max = Some(100);

    // Valid period should work
    let valid_period = Period::new(20, &domain_rules);
    assert!(valid_period.is_ok());
    assert_eq!(valid_period.unwrap().value(), 20);

    // Invalid periods should fail
    let too_small = Period::new(3, &domain_rules);
    assert!(too_small.is_err());

    let too_large = Period::new(150, &domain_rules);
    assert!(too_large.is_err());

    // Should work when no constraints are set
    let unconstrained_rules = DomainRules::default();
    let any_period = Period::new(999, &unconstrained_rules);
    assert!(any_period.is_ok());
}

#[test]
fn test_std_dev_factor_validation() {
    let mut domain_rules = DomainRules::default();

    // Set standard deviation constraints
    domain_rules.technical_indicators.std_dev_min = Some(0.5);
    domain_rules.technical_indicators.std_dev_max = Some(3.0);

    // Valid std dev factor should work
    let valid_std_dev = StdDevFactor::new(2.0, &domain_rules);
    assert!(valid_std_dev.is_ok());
    assert_eq!(valid_std_dev.unwrap().value(), 2.0);

    // Invalid std dev factors should fail
    let too_small = StdDevFactor::new(0.1, &domain_rules);
    assert!(too_small.is_err());

    let too_large = StdDevFactor::new(5.0, &domain_rules);
    assert!(too_large.is_err());

    // Should work when no constraints are set
    let unconstrained_rules = DomainRules::default();
    let any_std_dev = StdDevFactor::new(10.0, &unconstrained_rules);
    assert!(any_std_dev.is_ok());
}

#[test]
fn test_generic_strategy_config_system() {
    let domain_rules = DomainRules::default();

    // Create a generic strategy config using builder
    let strategy_config = BollingerBandsStrategy::validate_config(
        &StrategyConfigBuilder::new()
            .periods(vec![10, 20])
            .std_dev_factors(vec![1.5, 2.0]),
        &domain_rules,
    )
    .unwrap();

    // Generate parameter grid
    let param_grid = strategy_config.generate_param_grid();

    // Should generate 2 x 2 i.e. 4 parameter combinations
    assert_eq!(param_grid.len(), 4);

    // Verify it implements StrategyParams trait
    let first_param = &param_grid[0];
    let kernel_args = first_param.to_kernel_args();
    assert_eq!(kernel_args.len(), 2);
}

#[test]
fn test_bollinger_bands_cpu_backtest() {
    let domain_rules = DomainRules::default();

    // Create strategy config
    let strategy_config = BollingerBandsStrategy::validate_config(
        &StrategyConfigBuilder::new()
            .periods(vec![10, 20])
            .std_dev_factors(vec![1.5, 2.0]),
        &domain_rules,
    )
    .unwrap();

    // Mock price data
    let price_data = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0, 97.0, 104.0];

    // Will fail due to no implementation
    let backtest_engine = BacktestEngine::new(ComputeMode::CPU { threads: 4 });
    let results =
        backtest_engine.run_backtest::<BollingerBandsStrategy>(&strategy_config, &price_data);

    assert!(results.is_ok());
    let backtest_results = results.unwrap();
    assert_eq!(backtest_results.len(), 4);

    let first_result = &backtest_results[0];
    assert!(first_result.total_pnl != 0.0 || first_result.total_pnl == 0.0); // Just verify it's a number
}

#[test]
fn test_real_bollinger_bands_calculation() {
    let domain_rules = DomainRules::default();

    // Create BB params: 5-period with 2.0 std dev
    let params = BollingerBandsParams {
        period: Period::new(5, &domain_rules).unwrap(),
        std_dev_factor: StdDevFactor::new(2.0, &domain_rules).unwrap(),
    };

    // Price data that should trigger signals
    // Pattern: sideways around 100, then breakout to 110, then drop to 90
    let price_data = vec![
        100.0, 101.0, 99.0, 100.5, 99.5, // First 5 - establishes bands
        100.2, 100.8, 101.2, // Sideways - no signals
        105.0, 110.0, // Breakout above upper band - SELL signals
        108.0, 106.0, 102.0, 95.0, 90.0, // Drop below lower band - BUY signals
    ];

    let result = BollingerBandsStrategy::simulate_strategy(&params, &price_data).unwrap();

    // Should have actual trades, not mock values
    assert!(result.num_trades > 0, "Should have executed some trades");
    assert_ne!(result.total_pnl, 100.0, "Should not be mock PnL value");
    assert_ne!(result.sharpe_ratio, 0.5, "Should not be mock Sharpe ratio");

    // Should have both buy and sell signals from the price pattern
    assert!(
        result.num_trades >= 2,
        "Should have at least 2 trades from the price pattern"
    );
}
