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

// #[test]
// fn test_generic_strategy_config_system() {
//     let domain_rules = DomainRules::default();

//     // Create a generic strategy config using builder
//     let strategy_config = StrategyConfigBuilder::new()
//         .periods(vec![10, 20])
//         .std_dev_factors(vec![1.5, 2.0])
//         .build::<BollingerBandsParams>(&domain_rules)
//         .unwrap();
// }
