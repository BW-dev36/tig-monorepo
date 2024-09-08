#pragma once

#ifdef __GPU__
    #include <cuda_runtime.h>
#endif

#include <stdint.h>

typedef uint32_t weight_t;
typedef uint32_t value_t;
typedef uint32_t index_t;

#define HOST_MAX_MEM 5368709120 // 5GiB
#define NUM_THREADS 384
#define NUM_SEGMENTS 256

#ifdef __GPU__
// GPU knapsack problem solver.
value_t gpu_knapsack(const weight_t capacity,
                     const weight_t* weights,
                     const value_t* values,
                     const index_t num_items,
                     uint32_t* taken_indices);
#endif

extern "C" {
    typedef struct {
        unsigned int max_weight;
        unsigned int min_value;
        unsigned int num_items;
        const unsigned int* values;
        const unsigned int* weights;
    } Challenge;

    typedef struct {
        int* items;
    } Solution;


    void solve_dynamic_cuda(Challenge challenge, Solution solution);

    void solve_knapmaxxing_v1_cuda(Challenge challenge, Solution solution);
    void solve_knapmaxxing_v2_cuda(Challenge challenge, Solution solution);

    void solve_knapmaxxing_cpp(Challenge &challenge, Solution &solution);
}
