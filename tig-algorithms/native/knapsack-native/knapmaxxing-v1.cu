#ifndef __CUDACC__
#define __CUDACC__
#endif
#include "cuda_runtime.h"
#include "device_launch_parameters.h"
#include <cuda.h>
#include <device_functions.h>
#include <cuda_runtime_api.h>
#include <vector>
#include <stdio.h>
#include <iostream>
#include <mutex>
#include <algorithm>
#include "dp_cuda.h"

std::mutex lock_check;
std::mutex race_cond;
//std::lock_guard<std::mutex> lock(lock_check); \

void knapsackCuda(unsigned int *output, const unsigned int *val, const unsigned int *wt, unsigned int W, int num_items, int *selected_indices);

__device__ int maxi(int a, int b) { 
	return (a > b)? a : b; 
}

// __global__ void knapsackKernel(unsigned int *wt, unsigned int *val, unsigned int *output, unsigned int W, int i) {
// 	int w = threadIdx.x;

// 	//__syncthreads();
// 	if (i == 0 || w == 0)
// 		output[(i*W)+w] = 0;
// 	else if (wt[i-1] <= w)
// 		output[(i*W)+w] = maxi(val[i-1] + output[((i-1)*W)+(w-wt[i-1])],  output[((i-1)*W)+w]);
//         printf("Index added %d", )
// 	else
// 		output[(i*W)+w] = output[((i-1)*W)+w];
// 	__syncthreads();
   
// }

__global__ void knapsackKernel(unsigned int *wt, unsigned int *val, unsigned int *output, unsigned int W, int i, int *selected_indices) {
    

    for (int w = threadIdx.x; w <= W; w += blockDim.x)
    {
        if (i == 0 || w == 0) {
            output[(i * (W + 1)) + w] = 0;
            selected_indices[(i * (W + 1)) + w] = -1; // Sentinel value to indicate no item selected
        } else if (wt[i - 1] <= w) {
            unsigned int include_item = val[i - 1] + output[((i - 1) * (W + 1)) + (w - wt[i - 1])];
            unsigned int exclude_item = output[((i - 1) * (W + 1)) + w];
            output[(i * (W + 1)) + w] = maxi(include_item, exclude_item);

            if (include_item > exclude_item) {
                // printf("Index to take : %d\n", i - 1);
                selected_indices[(i * (W + 1)) + w] = i - 1; // Store the index of the included item
            } else {
                selected_indices[(i * (W + 1)) + w] = selected_indices[((i - 1) * (W + 1)) + w]; // Propagate the index of the previous item
            }
        } else {
            output[(i * (W + 1)) + w] = output[((i - 1) * (W + 1)) + w];
            selected_indices[(i * (W + 1)) + w] = selected_indices[((i - 1) * (W + 1)) + w]; // Propagate the index of the previous item
        }
    }
    __syncthreads();
}


int get_nb_gpu()
{
    int nb_gpu = 0;
    cudaGetDeviceCount(&nb_gpu);
    return nb_gpu;
}

#include <vector>

extern "C" void solve_challenge_v1_cuda(Challenge challenge, Solution solution) {
    int n = challenge.num_items;
    int W = challenge.max_weight;
    const unsigned int *val = challenge.values;
    const unsigned int *wt = challenge.weights;
    unsigned int *output = 0;
    int *selected_indices = 0;

    output = (unsigned int *)malloc((n + 1) * (W + 1) * sizeof(unsigned int));
    selected_indices = (int *)malloc((n + 1) * (W + 1) * sizeof(int));

    // Create a vector of indices
    std::vector<unsigned int> indices(n);
    for (int i = 0; i < n; ++i) {
        indices[i] = i;
    }

    // Sort indices based on the ratio of value to weight
    std::sort(indices.begin(), indices.end(), [&](int a, int b) {
        return (float)val[a] / wt[a] > (float)val[b] / wt[b];
    });

    // Apply sorted indices to weights and values
    std::vector<unsigned int> sorted_weights(n);
    std::vector<unsigned int> sorted_values(n);
    for (int i = 0; i < n; ++i) {
        sorted_weights[i] = wt[indices[i]];
        sorted_values[i] = val[indices[i]];
    }
    val = sorted_values.data();
    wt = sorted_weights.data();
    // printf("min value %d | max weight %d\n", n, W);
    // printf("input values | ");
    // for (int i = 0; i < n; i++)
    //     printf("%d ", val[i]);
    // printf("|\n");
    // printf("input weight | ");
    // for (int i = 0; i < n; i++)
    //     printf("%d ", wt[i]);
    // printf("|\n");


    knapsackCuda(output, val, wt, W, n, selected_indices);

    // for (int i = 0; i <= n; i++)
	// 	for (int j = 0; j <= W; j++) {
	// 		std::cout << output[i*(W + 1) + j] << ";";
	// 		if (j == W)
	// 			std::cout << std::endl;
	// }

    //std::cout << "Maximum Value possible for knapsack with capacity " << W << " is : " << output[(n +1) * (W + 1) - 1] << std::endl;

    // Retrieve the indices of the selected items
    int currentW = W;
    int count = 0;

    std::vector<bool> taken(n, false);
    // printf("Last colomuns : ");

    // for (int i = 1; i <= n; i++) {
    //     int value = output[i * (W + 1) + W - 1] - output[(i - 1) * (W + 1) + W - 1];

    //     //Find value corresponding with the lightest w
    //     printf("%d ", value);
    // }
    // printf("\n");

    for (int i = n; i > 0 ; --i) {

        if ((i) * (W + 1) + currentW >= (n + 1) * (W + 1))
        {
            printf("Out of bound index (i) * (W + 1) + currentW\n");
        }

        int idx = selected_indices[(i) * (W + 1) + currentW];

        if (idx >= n)
        {
            printf("Out of bound index Idx : %d : %d %d \n", (i) * (W + 1) + currentW, i, currentW);
        }


        if (idx != -1 && !taken[idx]) {
            solution.items[count++] = indices[idx];
            taken[idx] = true;
            currentW -= wt[idx];
            if (currentW < 0) currentW = 0;
            // Update the indices to avoid duplicate selections
            for (int j = i - 1; j >= 0; --j) {
                
                if (selected_indices[j * (W + 1) + currentW] == idx) {
                    selected_indices[j * (W + 1) + currentW] = -1;
                }
            }
        }
        // else if (idx != -1 && taken[idx]) // Find the next one
        // {
        //     int value = val[idx];
        //     int int_max = 99999;
        //     int best_weight = int_max;
        //     int best_index  = int_max;
        //     for (int k = 0; k < n; k++)
        //     {
        //         if (value == val[idx] && !taken[k] && best_weight > wt[idx])
        //         {
        //             best_index = k;
        //             best_weight = wt[idx];
        //             taken[best_index] = true;
        //             solution.items[count++] = best_index;
        //             currentW -= wt[best_index];
        //             break;
        //         }
        //     }
        //     // if (best_index != int_max)
        //     // {
        //     //     taken[best_index] = true;
        //     //     solution.items[count++] = best_index;
        //     //     currentW -= wt[best_index];
        //     // }
        // }
    }


    // //Print indices and calculate final value and weight
    // printf("Indices |\n");
    // int final_value = 0;
    // int final_weight = 0;
    // for (int i = 0; i < count; ++i) {
    //     int idx = solution.items[i];
    //     final_value += val[idx];
    //     final_weight += wt[idx];
    //     printf("Indice = %d, (%d %d) => Sum => %d %d\n", idx, val[idx], wt[idx], final_value, final_weight);
    // }

    // if (final_value != output[n * (W + 1) + W]) {
    //     printf("Invalid solution: Total value (%d) does not reach maximum value (%d)\n", final_value, output[(n + 1) * (W + 1)]);
    // }

    // if (final_weight > W) {
    //     printf("Invalid solution: Total weight (%d) exceeded max weight (%d)\n", final_weight, W);
    // }


    free(output);
    free(selected_indices);

}

#include <thread>
#include <atomic>
// Global variable to track the next GPU to assign
std::atomic<unsigned int> next_gpu_index(0);

void knapsackCuda(unsigned int *output, const unsigned int *val, const unsigned int *wt, unsigned int W, int num_items, int *selected_indices) {
    unsigned int *dev_val = 0;
    unsigned int *dev_wt = 0;
    unsigned int *dev_output = 0;
    int *dev_selected_indices = 0;

    std::lock_guard<std::mutex> lock(lock_check);
	// cudaEvent_t start, stop;
	// cudaEventCreate(&start);
	// cudaEventCreate(&stop);
    //std::lock_guard<std::mutex> lock(local_guard);
    unsigned int gpu_to_use = (next_gpu_index++) % get_nb_gpu();
    CUDA_CHECK(cudaSetDevice(gpu_to_use));

    cudaStream_t stream;
    cudaStreamCreate(&stream);

    CUDA_CHECK(cudaMalloc((void**)&dev_output, (num_items + 1) * (W + 1) * sizeof(unsigned int)));
    

    CUDA_CHECK(cudaMalloc((void**)&dev_val, num_items * sizeof(unsigned int)));
    

    CUDA_CHECK(cudaMalloc((void**)&dev_wt, num_items * sizeof(unsigned int)));
    
    
    CUDA_CHECK(cudaMalloc((void**)&dev_selected_indices, (num_items + 1) * (W + 1) * sizeof(int)));
    

    CUDA_CHECK(cudaMemcpy(dev_val, val, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice));

    CUDA_CHECK(cudaMemcpy(dev_wt, wt, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice));

	//cudaEventRecord(start);
	// Determine number of threads per block
    int threadsPerBlock = 128;

    // Launch a kernel on the GPU with one block per iteration
    {
        //std::cout << "Thread Id : " << std::this_thread::get_id() << std::endl;     
        for (int i = 0; i <= num_items; i++) {
            knapsackKernel<<<1, threadsPerBlock,0, stream>>>(dev_wt, dev_val, dev_output, W, i, dev_selected_indices);
        }
    }
    CUDA_CHECK(cudaStreamSynchronize(stream));

    // CUDA_CHECK(cudaDeviceSynchronize());
	//cudaEventRecord(stop);

    CUDA_CHECK(cudaMemcpy(output, dev_output, (num_items + 1) * (W + 1) * sizeof(unsigned int), cudaMemcpyDeviceToHost));

    // Copy selected indices from GPU buffer to host memory.
    CUDA_CHECK(cudaMemcpy(selected_indices, dev_selected_indices, (num_items + 1) * (W + 1) * sizeof(int), cudaMemcpyDeviceToHost));

	//cudaEventSynchronize(stop);
	//float milliseconds = 0;
	//cudaEventElapsedTime(&milliseconds, start, stop);

	//std::cout << "Execution Time : " << milliseconds / 1000 << " seconds" << std::endl;

    CUDA_CHECK(cudaFree(dev_output));
    CUDA_CHECK(cudaFree(dev_val));
    CUDA_CHECK(cudaFree(dev_wt));
    CUDA_CHECK(cudaFree(dev_selected_indices));
    CUDA_CHECK(cudaStreamDestroy(stream));
}