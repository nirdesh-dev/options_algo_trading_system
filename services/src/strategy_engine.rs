pub fn new_strategy_engine(_strategy: ()) -> () {
    ()
}

pub fn new_bollinger_bands_strategy(_period: usize, _std_dev_factor: f64) -> () {
    ()
}

#[cfg(test)]
#[path = "tests/strategy_engine_test.rs"]
mod strategy_engine_test;
