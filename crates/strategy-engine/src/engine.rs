use crate::traits::Strategy;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use domain::domain::{Quote, Signal};
use std::collections::HashMap;

pub struct StrategyEngine {
    strategies: HashMap<String, Box<dyn Strategy>>,
    signal_sender: Sender<Signal>,
    signal_receiver: Receiver<Signal>,
}

impl StrategyEngine {
    pub fn new() -> Self {
        let (signal_sender, signal_receiver) = crossbeam_channel::unbounded();

        Self {
            strategies: HashMap::new(),
            signal_sender,
            signal_receiver,
        }
    }

    /// Add a strategy to the engine
    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        let strategy_id = strategy.get_id().to_string();
        eprintln!(
            "Added strategy: {} (symbols: {:?})",
            strategy_id,
            strategy.get_required_symbols()
        );
        self.strategies.insert(strategy_id, strategy);
    }

    /// Remove a strategy from the engine
    pub fn remove_strategy(&mut self, strategy_id: &str) -> Option<Box<dyn Strategy>> {
        self.strategies.remove(strategy_id)
    }

    /// Process a single quote through all strategies
    pub fn process_quote(&mut self, quote: &Quote) -> Result<()> {
        for (name, strategy) in self.strategies.iter_mut() {
            if strategy.get_required_symbols().contains(&quote.symbol) {
                if let Some(signal) = strategy.process_quote(quote) {
                    eprintln!("Strategy {} generated signal: {:?}", name, signal);
                    self.signal_sender.send(signal)?;
                }
            }
        }
        Ok(())
    }

    /// Start processing quotes from a receiver in a background thread
    pub fn start_processing(mut self, quote_receiver: Receiver<Quote>) -> Receiver<Signal> {
        let signal_receiver = self.signal_receiver.clone();
        std::thread::spawn(move || {
            eprintln!("Strategy engine started processing quotes");
            while let Ok(quote) = quote_receiver.recv() {
                if let Err(e) = self.process_quote(&quote) {
                    eprintln!("Error processing quote: {}", e);
                }
            }
            eprintln!("Strategy engine stopped processing quotes")
        });
        signal_receiver
    }

    /// Get signal receiver for manual processing
    pub fn get_signal_receiver(&self) -> Receiver<Signal> {
        self.signal_receiver.clone()
    }

    /// Get list of active strategies
    pub fn get_active_strategies(&self) -> Vec<String> {
        self.strategies.keys().cloned().collect()
    }

    /// Get required symbols from all strategies
    pub fn get_required_symbols(&self) -> Vec<String> {
        let mut symbols = std::collections::HashSet::new();

        for strategy in self.strategies.values() {
            for symbol in strategy.get_required_symbols() {
                symbols.insert(symbol.clone());
            }
        }

        symbols.into_iter().collect()
    }
}
