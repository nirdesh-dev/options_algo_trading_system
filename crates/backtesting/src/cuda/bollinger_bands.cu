extern "C" __global__ void bollinger_bands_kernel(
    const float *price_data,
    int price_count,
    const float *periods,
    const float *std_dev_factors,
    int param_count,
    float *pnl,
    float *num_trades,
    float *sharpe_ratio,
    float *max_drawdown,
    float *win_rate,      // Add win_rate output
    float *volatility     // Add volatility output
)
{
    int param_idx = blockIdx.x * blockDim.x + threadIdx.x;

    if (param_idx >= param_count)
        return;

    // Extract parameters for this thread
    int period = (int)periods[param_idx];
    float std_dev_factor = std_dev_factors[param_idx];

    // Trading state
    float position = 0.0f;
    float entry_price = 0.0f;
    float total_pnl = 0.0f;
    int trades = 0;
    int winning_trades = 0;  // Track winning trades
    float max_dd = 0.0f;
    float peak_pnl = 0.0f;

    // Track returns for Sharpe ratio and volatility
    float returns_sum = 0.0f;
    float returns_sq_sum = 0.0f;
    int return_count = 0;

    // Process price data starting from period
    for (int i = period; i < price_count; i++)
    {
        float current_price = price_data[i];

        // Calculate SMA for current window
        float sma = 0.0f;
        for (int j = i - period; j < i; j++)
        {
            sma += price_data[j];
        }
        sma /= period;

        // Calculate standard deviation
        float variance = 0.0f;
        for (int j = i - period; j < i; j++)
        {
            float diff = price_data[j] - sma;
            variance += diff * diff;
        }
        variance /= period;
        float std_dev = sqrtf(variance);

        // Calculate Bollinger Bands
        float upper_band = sma + (std_dev_factor * std_dev);
        float lower_band = sma - (std_dev_factor * std_dev);

        // Trading logic
        if (position == 0.0f)
        {
            // No position - look for entry signals
            if (current_price <= lower_band)
            {
                // BUY signal
                position = 1.0f;
                entry_price = current_price;
            }
            else if (current_price >= upper_band)
            {
                // SELL signal
                position = -1.0f;
                entry_price = current_price;
            }
        }
        else
        {
            // Have position - look for exit signals
            if (position > 0.0f && current_price >= sma)
            {
                // Exit long position
                float trade_return = (current_price - entry_price) / entry_price;
                total_pnl += trade_return;
                returns_sum += trade_return;
                returns_sq_sum += trade_return * trade_return;
                return_count++;
                trades++;
                
                // Track winning trades
                if (trade_return > 0.0f) {
                    winning_trades++;
                }
                
                position = 0.0f;

                // Update drawdown tracking
                if (total_pnl > peak_pnl)
                    peak_pnl = total_pnl;
                float drawdown = peak_pnl - total_pnl;
                if (drawdown > max_dd)
                    max_dd = drawdown;
            }
            else if (position < 0.0f && current_price <= sma)
            {
                // Exit short position
                float trade_return = (entry_price - current_price) / entry_price;
                total_pnl += trade_return;
                returns_sum += trade_return;
                returns_sq_sum += trade_return * trade_return;
                return_count++;
                trades++;
                
                // Track winning trades
                if (trade_return > 0.0f) {
                    winning_trades++;
                }
                
                position = 0.0f;

                // Update drawdown tracking
                if (total_pnl > peak_pnl)
                    peak_pnl = total_pnl;
                float drawdown = peak_pnl - total_pnl;
                if (drawdown > max_dd)
                    max_dd = drawdown;
            }
        }
    }

    // Calculate metrics
    float sharpe = 0.0f;
    float win_rate_val = 0.0f;
    float volatility_val = 0.0f;
    
    if (return_count > 1)
    {
        float mean_return = returns_sum / return_count;
        float variance_return = (returns_sq_sum / return_count) - (mean_return * mean_return);
        float std_return = sqrtf(variance_return);
        
        if (std_return > 0.0f)
        {
            sharpe = mean_return / std_return;
        }
        
        volatility_val = std_return;  // Volatility is standard deviation of returns
    }
    
    if (trades > 0)
    {
        win_rate_val = (float)winning_trades / (float)trades;
    }

    // Store results in separate arrays
    pnl[param_idx] = total_pnl;
    num_trades[param_idx] = (float)trades;
    sharpe_ratio[param_idx] = sharpe;
    max_drawdown[param_idx] = -max_dd;
    win_rate[param_idx] = win_rate_val;
    volatility[param_idx] = volatility_val;
}
