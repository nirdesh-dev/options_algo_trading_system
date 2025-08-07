use super::*;

#[test]
fn test_can_create_strategy_engine() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);
}

#[test]
fn test_strategy_engine_has_subscribe_method() {
    let strategy = new_bollinger_bands_strategy(20, 2.0);
    let engine = new_strategy_engine(strategy);

    let _receiver = engine.subscribe_signals();
}
