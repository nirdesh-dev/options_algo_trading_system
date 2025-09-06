use domain::domain::{Quote, Signal};

pub trait Strategy: Send + Sync {
    /// Process a quote and optionally generate a signal
    fn process_quote(&mut self, quote: &Quote) -> Option<Signal>;

    /// Get the strategy's unique identifier
    fn get_id(&self) -> &str;

    /// Get symbols this strategy is interested in
    fn get_required_symbols(&self) -> &[String];

    /// Reset strategy state (useful for backtesting)
    fn reset(&mut self);
}
