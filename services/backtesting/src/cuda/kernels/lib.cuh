// src/kernel/lib.cuh
#ifndef LIB_CUH
#define LIB_CUH

// Pricing model identifiers
#define PRICING_BLACK_SCHOLES 0

// Call/Put flags
#define OPTION_CALL 0
#define OPTION_PUT 1

__device__ double erf_approx(double x)
{
    // Basic erf approximation (for CUDA device)
    // Abramowitz and Stegun formula 7.1.26
    double t = 1.0 / (1.0 + 0.5 * fabs(x));
    double tau = t * exp(-x * x - 1.26551223 +
                         t * (1.00002368 +
                              t * (0.37409196 +
                                   t * (0.09678418 +
                                        t * (-0.18628806 +
                                             t * (0.27886807 +
                                                  t * (-1.13520398 +
                                                       t * (1.48851587 +
                                                            t * (-0.82215223 +
                                                                 t * 0.17087277)))))))));
    return (x >= 0.0) ? (1.0 - tau) : (tau - 1.0);
}

__device__ double normal_cdf(double x)
{
    return 0.5 * (1.0 + erf_approx(x / sqrt(2.0)));
}

#endif // LIB_CUH
