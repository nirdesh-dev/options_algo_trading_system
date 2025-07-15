#ifndef PRICING_BLACK_SCHOLES_CUH
#define PRICING_BLACK_SCHOLES_CUH

#include "lib.cuh"

__device__ float price_black_scholes(
    float S, float K, float T,
    float r, float sigma,
    int call_or_put)
{
    float d1 = (logf(S / K) + (r + 0.5f * sigma * sigma) * T) / (sigma * sqrtf(T));
    float d2 = d1 - sigma * sqrtf(T);

    if (call_or_put == OPTION_CALL)
    {
        return S * normal_cdf(d1) - K * expf(-r * T) * normal_cdf(d2);
    }
    else
    {
        return K * expf(-r * T) * normal_cdf(-d2) - S * normal_cdf(-d1);
    }
}

#endif // PRICING_BLACK_SCHOLES_CUH
