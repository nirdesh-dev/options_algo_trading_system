use anyhow::{Context, Result, anyhow};
use chrono::NaiveDate;
use polars::{functions::concat_df_diagonal, prelude::*};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub struct OptionsData {
    pub data: DataFrame,
}

#[derive(Debug, Clone)]
pub struct StocksData {
    pub data: DataFrame,
}

#[derive(Debug, Clone)]
pub struct BacktestData {
    pub joined_data: DataFrame,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataLoaderConfig {
    pub data_dir: String,
    pub underlying_symbols: Vec<String>,
    pub option_types: Option<Vec<String>>, // "call", "put", or both
    pub date_range: Option<DateRange>,
    pub strike_range: Option<StrikeRange>,
    pub volume_filter: Option<VolumeFilter>,
    pub open_interest_filter: Option<OpenInterestFilter>,
    pub greeks_filter: Option<GreeksFilter>,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            data_dir: "./".to_string(),
            underlying_symbols: vec!["SPX".to_string()],
            date_range: None,
            option_types: None,
            strike_range: None,
            volume_filter: None,
            greeks_filter: None,
            open_interest_filter: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeFilter {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenInterestFilter {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeksFilter {
    pub delta_range: Option<DeltaRange>,
    pub gamma_range: Option<GammaRange>,
    pub theta_range: Option<ThetaRange>,
    pub vega_range: Option<VegaRange>,
    pub iv_range: Option<IvRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThetaRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegaRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DataLoaderConfig {
    /// Load configuration from YAML file
    pub fn from_yaml(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;

        let config: DataLoaderConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML config: {}", path))?;

        config.validate()?; // Enforced here

        Ok(config)
    }

    /// Save configuration to YAML file
    pub fn to_yaml(&self, path: &str) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize config to YAML")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write config to file: {}", path))?;

        Ok(())
    }

    /// Validates the entire data loader configuration.
    pub fn validate(&self) -> Result<()> {
        if !Path::new(&self.data_dir).exists() {
            return Err(anyhow!("Data directory does not exist: {}", self.data_dir));
        }

        if self.underlying_symbols.is_empty() {
            return Err(anyhow!("At least one underlying symbol must be specified"));
        }

        self.validate_date_range()?;
        self.validate_option_types()?;
        self.validate_strike_range()?;
        self.validate_liquidity_filters()?;
        self.validate_greeks_filters()?;

        Ok(())
    }

    // --- Private Validation Helpers ---

    fn validate_date_range(&self) -> Result<()> {
        if let Some(ref range) = self.date_range {
            let start_date =
                NaiveDate::parse_from_str(&range.start, "%Y-%m-%d").with_context(|| {
                    format!(
                        "Invalid start date format: {}. Use YYYY-MM-DD.",
                        range.start
                    )
                })?;
            let end_date =
                NaiveDate::parse_from_str(&range.end, "%Y-%m-%d").with_context(|| {
                    format!("Invalid end date format: {}. Use YYYY-MM-DD.", range.end)
                })?;

            if start_date >= end_date {
                return Err(anyhow!("Start date must be before end date."));
            }
        }
        Ok(())
    }

    fn validate_option_types(&self) -> Result<()> {
        if let Some(ref types) = self.option_types {
            for t in types {
                if t.to_lowercase() != "call" && t.to_lowercase() != "put" {
                    return Err(anyhow!(
                        "Invalid option type: '{}'. Must be 'call' or 'put'.",
                        t
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_strike_range(&self) -> Result<()> {
        if let Some(ref range) = self.strike_range {
            if range.min <= 0.0 || range.max <= 0.0 {
                return Err(anyhow!("Strike prices in strike_range must be positive."));
            }
            if range.min >= range.max {
                return Err(anyhow!("strike_range min must be less than max."));
            }
        }
        Ok(())
    }

    fn validate_liquidity_filters(&self) -> Result<()> {
        if let Some(ref filter) = self.volume_filter {
            if let (Some(min), Some(max)) = (filter.min, filter.max) {
                if min >= max {
                    return Err(anyhow!("Volume filter min must be less than max."));
                }
            }
        }
        if let Some(ref filter) = self.open_interest_filter {
            if let (Some(min), Some(max)) = (filter.min, filter.max) {
                if min >= max {
                    return Err(anyhow!("Open interest filter min must be less than max."));
                }
            }
        }
        Ok(())
    }

    fn validate_greeks_filters(&self) -> Result<()> {
        if let Some(ref greeks) = self.greeks_filter {
            if let Some(ref r) = greeks.delta_range {
                if let (Some(min), Some(max)) = (r.min, r.max) {
                    if min >= max {
                        return Err(anyhow!("Delta range min must be less than max."));
                    }
                }
            }
            if let Some(ref r) = greeks.gamma_range {
                if let (Some(min), Some(max)) = (r.min, r.max) {
                    if min >= max {
                        return Err(anyhow!("Gamma range min must be less than max."));
                    }
                }
            }
            if let Some(ref r) = greeks.theta_range {
                if let (Some(min), Some(max)) = (r.min, r.max) {
                    if min >= max {
                        return Err(anyhow!("Theta range min must be less than max."));
                    }
                }
            }
            if let Some(ref r) = greeks.vega_range {
                if let (Some(min), Some(max)) = (r.min, r.max) {
                    if min >= max {
                        return Err(anyhow!("Vega range min must be less than max."));
                    }
                }
            }
            if let Some(ref r) = greeks.iv_range {
                if let (Some(min), Some(max)) = (r.min, r.max) {
                    if min >= max {
                        return Err(anyhow!("IV range min must be less than max."));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLoader {
    config: DataLoaderConfig,
}

impl DataLoader {
    pub fn new(config: DataLoaderConfig) -> Result<Self> {
        Ok(Self { config })
    }

    pub fn from_yaml(config_path: &str) -> Result<Self> {
        let config = DataLoaderConfig::from_yaml(config_path)?;
        Ok(Self { config })
    }

    fn apply_options_filters(&self, lazy_frame: LazyFrame) -> LazyFrame {
        let mut filtered = lazy_frame;
        if !self.config.underlying_symbols.is_empty() {
            let symbols_series = Series::new("symbols".into(), &self.config.underlying_symbols);
            filtered = filtered.filter(col("underlying").is_in(lit(symbols_series), false));
        }

        if let Some(ref types) = self.config.option_types {
            if !types.is_empty() {
                let option_types_series = Series::new("types".into(), types);
                filtered = filtered.filter(col("type").is_in(lit(option_types_series), false));
            }
        }

        if let Some(ref strike_range) = self.config.strike_range {
            filtered = filtered.filter(
                col("strike")
                    .gt_eq(lit(strike_range.min))
                    .and(col("strike").lt_eq(lit(strike_range.max))),
            );
        }

        if let Some(ref volume_filter) = self.config.volume_filter {
            if let Some(min) = volume_filter.min {
                filtered = filtered.filter(col("volume").gt_eq(lit(min)));
            }
            if let Some(max) = volume_filter.max {
                filtered = filtered.filter(col("volume").lt_eq(lit(max)));
            }
        }

        if let Some(ref oi_filter) = self.config.open_interest_filter {
            if let Some(min) = oi_filter.min {
                filtered = filtered.filter(col("open_interest").gt_eq(lit(min)));
            }
            if let Some(max) = oi_filter.max {
                filtered = filtered.filter(col("open_interest").lt_eq(lit(max)));
            }
        }

        if let Some(ref greeks) = self.config.greeks_filter {
            if let Some(ref r) = greeks.delta_range {
                if let Some(min) = r.min {
                    filtered = filtered.filter(col("delta").gt_eq(lit(min)));
                }
                if let Some(max) = r.max {
                    filtered = filtered.filter(col("delta").lt_eq(lit(max)));
                }
            }
            if let Some(ref r) = greeks.gamma_range {
                if let Some(min) = r.min {
                    filtered = filtered.filter(col("gamma").gt_eq(lit(min)));
                }
                if let Some(max) = r.max {
                    filtered = filtered.filter(col("gamma").lt_eq(lit(max)));
                }
            }
            if let Some(ref r) = greeks.theta_range {
                if let Some(min) = r.min {
                    filtered = filtered.filter(col("theta").gt_eq(lit(min)));
                }
                if let Some(max) = r.max {
                    filtered = filtered.filter(col("theta").lt_eq(lit(max)));
                }
            }
            if let Some(ref r) = greeks.vega_range {
                if let Some(min) = r.min {
                    filtered = filtered.filter(col("vega").gt_eq(lit(min)));
                }
                if let Some(max) = r.max {
                    filtered = filtered.filter(col("vega").lt_eq(lit(max)));
                }
            }
            if let Some(ref r) = greeks.iv_range {
                if let Some(min) = r.min {
                    filtered = filtered.filter(col("implied_volatility").gt_eq(lit(min)));
                }
                if let Some(max) = r.max {
                    filtered = filtered.filter(col("implied_volatility").lt_eq(lit(max)));
                }
            }
        }

        filtered
    }

    /// Apply date range filter to LazyFrame
    fn apply_date_filter(&self, lazy_frame: LazyFrame) -> LazyFrame {
        if let Some(ref date_range) = self.config.date_range {
            lazy_frame.filter(
                col("date")
                    .gt_eq(lit(date_range.start.clone()))
                    .and(col("date").lt_eq(lit(date_range.end.clone()))),
            )
        } else {
            lazy_frame
        }
    }

    /// Defines the schema for the options CSV files.
    /// Defines the schema for the options CSV files.
    fn get_options_schema(&self) -> Schema {
        Schema::from_iter(vec![
            Field::new("contract".into(), DataType::String),
            Field::new("underlying".into(), DataType::String),
            Field::new("expiration".into(), DataType::String),
            Field::new("type".into(), DataType::String),
            Field::new("strike".into(), DataType::Float64),
            Field::new("style".into(), DataType::String),
            Field::new("bid".into(), DataType::Float64),
            Field::new("bid_size".into(), DataType::Int64),
            Field::new("ask".into(), DataType::Float64),
            Field::new("ask_size".into(), DataType::Int64),
            Field::new("volume".into(), DataType::Int64),
            Field::new("open_interest".into(), DataType::Int64),
            Field::new("quote_date".into(), DataType::String),
            Field::new("delta".into(), DataType::Float64),
            Field::new("gamma".into(), DataType::Float64),
            Field::new("theta".into(), DataType::Float64),
            Field::new("vega".into(), DataType::Float64),
            Field::new("implied_volatility".into(), DataType::Float64),
        ])
    }

    /// Defines the schema for the stocks CSV files.
    fn get_stocks_schema(&self) -> Schema {
        Schema::from_iter(vec![
            Field::new("symbol".into(), DataType::String),
            Field::new("open".into(), DataType::Float64),
            Field::new("high".into(), DataType::Float64),
            Field::new("low".into(), DataType::Float64),
            Field::new("close".into(), DataType::Float64),
            Field::new("volume".into(), DataType::Int64),
            Field::new("adjust_close".into(), DataType::Float64),
        ])
    }

    pub fn load_options_data(&self) -> Result<OptionsData> {
        let mut all_options = Vec::new();
        let entries = fs::read_dir(&self.config.data_dir)?;
        let schema = self.get_options_schema();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with("options.csv") {
                    let date_str = filename.replace("options.csv", "");
                    let lf = LazyCsvReader::new(&path)
                        .with_schema(Some(Arc::new(schema.clone())))
                        .with_ignore_errors(true)
                        .finish()?;

                    let filtered = self.apply_options_filters(lf);

                    let df = filtered
                        .with_columns([
                            lit(date_str.clone()).alias("date"),
                            col("underlying").alias("symbol"),
                            col("expiration").alias("expiry"),
                            col("type").alias("option_type"),
                        ])
                        .select([
                            col("date"),
                            col("symbol"),
                            col("contract"),
                            col("expiry"),
                            col("option_type"),
                            col("strike"),
                            col("bid"),
                            col("ask"),
                            col("volume"),
                            col("open_interest"),
                            col("bid_size"),
                            col("ask_size"),
                            col("style"),
                            col("quote_date"),
                            col("delta"),
                            col("gamma"),
                            col("theta"),
                            col("vega"),
                            col("implied_volatility"),
                        ]);

                    let df = self.apply_date_filter(df).collect()?;
                    if df.height() > 0 {
                        all_options.push(df);
                    }
                }
            }
        }

        if all_options.is_empty() {
            return Err(anyhow!("No options data found matching filters"));
        }

        let combined_df = concat_df_diagonal(&all_options)?;

        Ok(OptionsData { data: combined_df })
    }

    pub fn load_stocks_data(&self) -> Result<StocksData> {
        let mut all_stocks = Vec::new();
        let schema = self.get_stocks_schema();

        println!(
            "Loading stocks data for underlying symbols: {:?}",
            self.config.underlying_symbols
        );

        // Get all stocks files
        let entries = fs::read_dir(&self.config.data_dir)
            .with_context(|| format!("Failed to read data directory: {}", self.config.data_dir))?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with("stocks.csv") {
                    println!("Loading stocks file: {}", filename);

                    // Extract date from filename (e.g., "2013-01-02stocks.csv")
                    let date_str = filename.replace("stocks.csv", "");

                    let symbols = Series::new("symbols".into(), &self.config.underlying_symbols);

                    let lazy_frame = LazyCsvReader::new(&path)
                        .with_schema(Some(Arc::new(schema.clone())))
                        .with_ignore_errors(true)
                        .finish()
                        .context("Failed to scan CSV file")?
                        .filter(col("symbol").is_in(lit(symbols), false));

                    let df = lazy_frame
                        .with_columns([
                            // Add the date column from filename
                            lit(date_str.clone()).alias("date"),
                        ])
                        .select([
                            col("date"),
                            col("symbol"),
                            col("open"),
                            col("high"),
                            col("low"),
                            col("close"),
                            col("volume"),
                            col("adjust_close"),
                        ]);

                    // Apply date filter
                    let df = self
                        .apply_date_filter(df)
                        .collect()
                        .context("Failed to collect filtered data")?;

                    if df.height() > 0 {
                        all_stocks.push(df);
                    }
                }
            }
        }

        if all_stocks.is_empty() {
            return Err(anyhow!("No stocks data found matching filters"));
        }

        let combined_stocks = concat_df_diagonal(&all_stocks)?;
        Ok(StocksData {
            data: combined_stocks,
        })
    }

    pub fn join_data(&self, options: &OptionsData, stocks: &StocksData) -> Result<BacktestData> {
        let stocks_for_join = stocks.data.clone().lazy().select([
            col("date"),
            col("symbol").alias("underlying_symbol"),
            col("open").alias("underlying_open"),
            col("high").alias("underlying_high"),
            col("low").alias("underlying_low"),
            col("close").alias("underlying_close"),
            col("volume").alias("underlying_volume"),
            col("adjust_close").alias("underlying_adjust_close"),
        ]);

        let joined = options
            .data
            .clone()
            .lazy()
            .join(
                stocks_for_join,
                [col("date"), col("symbol")],
                [col("date"), col("underlying_symbol")],
                JoinArgs::new(JoinType::Inner),
            )
            .collect()
            .context("Failed to join options and stocks data")?;

        Ok(BacktestData {
            joined_data: joined,
        })
    }

    /// Complete data loading pipeline
    pub fn load_all_data(&self) -> Result<BacktestData> {
        let options = self
            .load_options_data()
            .context("Failed to load options data")?;
        println!("Loaded {} options records", options.data.height());

        let stocks = self
            .load_stocks_data()
            .context("Failed to load stocks data")?;
        println!("Loaded {} stocks records", stocks.data.height());

        println!("Joining data...");
        let backtest_data = self
            .join_data(&options, &stocks)
            .context("Failed to join data")?;
        println!(
            "Joined dataset has {} records",
            backtest_data.joined_data.height()
        );
        Ok(backtest_data)
    }
}

impl BacktestData {
    /// Filter data for specific date range
    pub fn filter_date_range(&self, start_date: &str, end_date: &str) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .filter(
                col("date")
                    .gt_eq(lit(start_date))
                    .and(col("date").lt_eq(lit(end_date))),
            )
            .collect()
            .context("Failed to filter by date range")
    }

    /// Filter for specific option types (call/put)
    pub fn filter_option_type(&self, option_type: &str) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .filter(col("option_type").eq(lit(option_type)))
            .collect()
            .context("Failed to filter by option type")
    }

    /// Filter for specific strike range
    pub fn filter_strike_range(&self, min_strike: f64, max_strike: f64) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .filter(
                col("strike")
                    .gt_eq(lit(min_strike))
                    .and(col("strike").lt_eq(lit(max_strike))),
            )
            .collect()
            .context("Failed to filter by strike range")
    }

    /// Calculate mid price for options
    pub fn with_mid_price(&self) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .with_columns([
                ((col("bid") + col("ask")) / lit(2.0)).alias("mid_price"),
                (col("ask") - col("bid")).alias("bid_ask_spread"),
            ])
            .collect()
            .context("Failed to calculate mid price")
    }

    /// Calculate moneyness (strike / underlying)
    pub fn with_moneyness(&self) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .with_columns([
                (col("strike") / col("underlying_close")).alias("moneyness"),
                (col("underlying_close") - col("strike")).alias("intrinsic_value_call"),
                (col("strike") - col("underlying_close")).alias("intrinsic_value_put"),
            ])
            .collect()
            .context("Failed to calculate moneyness")
    }

    /// Get summary statistics
    pub fn describe(&self) -> Result<DataFrame> {
        self.joined_data
            .clone()
            .lazy()
            .select([
                col("strike").mean().alias("avg_strike"),
                col("bid").mean().alias("avg_bid"),
                col("ask").mean().alias("avg_ask"),
                col("volume").mean().alias("avg_volume"),
                col("open_interest").mean().alias("avg_open_interest"),
                col("underlying_close").mean().alias("avg_underlying_close"),
                col("delta").mean().alias("avg_delta"),
                col("gamma").mean().alias("avg_gamma"),
                col("theta").mean().alias("avg_theta"),
                col("vega").mean().alias("avg_vega"),
                col("implied_volatility").mean().alias("avg_iv"),
            ])
            .collect()
            .context("Failed to calculate summary statistics")
    }

    pub fn get_underlying_symbols(&self) -> Result<Vec<String>> {
        let symbols = self
            .joined_data
            .clone()
            .lazy()
            .select([col("symbol")])
            .unique(None, UniqueKeepStrategy::First)
            .collect()
            .context("Failed to get unique symbols")?;

        Ok(symbols
            .column("symbol")
            .context("Failed to get symbol column")?
            .str()
            .context("Failed to convert to utf8")?
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect())
    }
}
