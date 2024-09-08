#include <cuda_runtime.h>
#include <device_launch_parameters.h>
#include <iostream>
#include <algorithm>
#include <vector>
#include "utils.h"
#include "dp_cuda.h"

__device__ float calculate_ratio(unsigned int value, unsigned int weight) {
    return static_cast<float>(value) / static_cast<float>(weight);
}

__global__ void sort_items_kernel(unsigned int* d_values, unsigned int* d_weights, int* d_indices, int num_items) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < num_items - 1) {
        float ratio_a = calculate_ratio(d_values[d_indices[idx]], d_weights[d_indices[idx]]);
        float ratio_b = calculate_ratio(d_values[d_indices[idx+1]], d_weights[d_indices[idx+1]]);
        if (ratio_b > ratio_a) {
            int temp = d_indices[idx];
            d_indices[idx] = d_indices[idx+1];
            d_indices[idx+1] = temp;
        }
    }
}

__global__ void solve_knapsack_kernel(unsigned int* d_values, unsigned int* d_weights, int* d_indices, 
                                      int* d_solution, unsigned int max_weight, unsigned int min_value, 
                                      int num_items, int* d_found_solution) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < (1 << num_items)) {
        unsigned int current_weight = 0;
        unsigned int current_value = 0;
        for (int i = 0; i < num_items; ++i) {
            if (idx & (1 << i)) {
                int item_idx = d_indices[i];
                current_weight += d_weights[item_idx];
                current_value += d_values[item_idx];
            }
        }
        if (current_weight <= max_weight && current_value >= min_value) {
            atomicExch(d_found_solution, 1);
            for (int i = 0; i < num_items; ++i) {
                if (idx & (1 << i)) {
                    printf("Solution Found !!");
                    d_solution[d_indices[i]] = 1;
                } else {
                    d_solution[d_indices[i]] = 0;
                }
            }
        }
    }
}

extern "C" void solve_dynamic_cuda(Challenge challenge, Solution solution) {
    int num_items = challenge.num_items;
    unsigned int max_weight = challenge.max_weight;
    unsigned int min_value = challenge.min_value;

    // Allocate device memory
    unsigned int *d_values, *d_weights;
    int *d_indices, *d_solution, *d_found_solution;

    CUDA_CHECK(cudaMalloc(&d_values, num_items * sizeof(unsigned int)));
    CUDA_CHECK(cudaMalloc(&d_weights, num_items * sizeof(unsigned int)));
    CUDA_CHECK(cudaMalloc(&d_indices, num_items * sizeof(int)));
    CUDA_CHECK(cudaMalloc(&d_solution, num_items * sizeof(int)));
    CUDA_CHECK(cudaMalloc(&d_found_solution, sizeof(int)));

    // Copy data to device
    CUDA_CHECK(cudaMemcpy(d_values, challenge.values, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice));
    CUDA_CHECK(cudaMemcpy(d_weights, challenge.weights, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice));

    // Initialize indices
    int* h_indices = new int[num_items];
    for (int i = 0; i < num_items; ++i) {
        h_indices[i] = i;
    }
    CUDA_CHECK(cudaMemcpy(d_indices, h_indices, num_items * sizeof(int), cudaMemcpyHostToDevice));
    delete[] h_indices;

    // Sort items by value-to-weight ratio
    int block_size = 256;
    int grid_size = (num_items + block_size - 1) / block_size;
    for (int i = 0; i < num_items; ++i) {
        sort_items_kernel<<<grid_size, block_size>>>(d_values, d_weights, d_indices, num_items);
        CUDA_CHECK(cudaDeviceSynchronize());
    }

    // Initialize solution
    CUDA_CHECK(cudaMemset(d_solution, 0, num_items * sizeof(int)));
    CUDA_CHECK(cudaMemset(d_found_solution, 0, sizeof(int)));

    // Solve knapsack problem
    int num_combinations = 1 << num_items;
    grid_size = (num_combinations + block_size - 1) / block_size;
    solve_knapsack_kernel<<<grid_size, block_size>>>(d_values, d_weights, d_indices, d_solution, 
                                                     max_weight, min_value, num_items, d_found_solution);
    CUDA_CHECK(cudaDeviceSynchronize());

    // Copy solution back to host
    CUDA_CHECK(cudaMemcpy(solution.items, d_solution, num_items * sizeof(int), cudaMemcpyDeviceToHost));

    // Clean up
    CUDA_CHECK(cudaFree(d_values));
    CUDA_CHECK(cudaFree(d_weights));
    CUDA_CHECK(cudaFree(d_indices));
    CUDA_CHECK(cudaFree(d_solution));
    CUDA_CHECK(cudaFree(d_found_solution));
}