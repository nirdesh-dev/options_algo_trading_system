#include "lib.cuh"
#include "black_scholes.cuh"

extern "C" __global__ void iron_condor_kernel(
    const float *price_series, int num_prices,
    const float *entry_times,
    const float *wing_widths,
    const float *stop_losses,
    int num_params,
    float *pnl,
    float *drawdown,
    float *sharpe,
    int pricing_model_flag)
{
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= num_params)
        return;

    float entry_time = entry_times[i];
    float wing_width = wing_widths[i];
    float stop_loss = stop_losses[i];

    float PNL = 0.0f;
    float max_dd = 0.0f;
    float sum_returns = 0.0f;
    float sum_squared = 0.0f;
    int count = 0;

    for (int t = 0; t < num_prices; ++t)
    {
        float S = price_series[t];
        float K_short_put = S - wing_width;
        float K_long_put = S - 2 * wing_width;
        float K_short_call = S + wing_width;
        float K_long_call = S + 2 * wing_width;
        float T = 1.0f;
        float r = 0.0f;
        float sigma = 0.2f;

        float short_put = 0.0f, long_put = 0.0f;
        float short_call = 0.0f, long_call = 0.0f;

        if (pricing_model_flag == PRICING_BLACK_SCHOLES)
        {
            short_put = price_black_scholes(S, K_short_put, T, r, sigma, OPTION_PUT);
            long_put = price_black_scholes(S, K_long_put, T, r, sigma, OPTION_PUT);
            short_call = price_black_scholes(S, K_short_call, T, r, sigma, OPTION_CALL);
            long_call = price_black_scholes(S, K_long_call, T, r, sigma, OPTION_CALL);
        }

        float net_credit = (short_put + short_call) - (long_put + long_call);
        float cumulative_pnl = net_credit;

        float max_loss = stop_loss * net_credit;
        if (cumulative_pnl < -max_loss)
        {
            cumulative_pnl = -max_loss;
        }

        PNL += cumulative_pnl;
        sum_returns += cumulative_pnl;
        sum_squared += cumulative_pnl * cumulative_pnl;
        max_dd = fminf(max_dd, cumulative_pnl);
        count++;
    }

    float mean = sum_returns / count;
    float stddev = sqrtf(fmaxf(1e-6f, (sum_squared / count) - (mean * mean)));
    float sharpe_ratio = mean / stddev;

    pnl[i] = PNL;
    drawdown[i] = max_dd;
    sharpe[i] = sharpe_ratio;
}
