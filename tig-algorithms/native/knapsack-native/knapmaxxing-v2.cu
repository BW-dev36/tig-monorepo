#ifndef __CUDACC__
#define __CUDACC__
#endif
#include "cuda_runtime.h"
#include "device_launch_parameters.h"
#include <cuda.h>
#include <cuda_runtime_api.h>
#include <vector>
#include <stdio.h>
#include <iostream>
#include <mutex>
#include <algorithm>
#include <thread>
#include <atomic>

#include "utils.h"

static std::mutex lock_check;
//std::lock_guard<std::mutex> lock(lock_check); \

#define GENERAL_MAX_WEIGHT 10000
#define GENERAL_MAX_NUM_ITEMS 150

// Global variable to track the next GPU to assign
static std::atomic<unsigned int> next_gpu_index(0);
static std::atomic<unsigned int> next_workspace_index(0);

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

    void solve_challenge_cuda(Challenge challenge, Solution solution);
}

class Workspace {
    public:
        unsigned int * dp;

        unsigned int * d_values;
        unsigned int * d_weights;
        unsigned int * d_indices;
        unsigned int * d_dp;
        int           num_items;
        int           max_weight;
        cudaStream_t stream;
        


        std::atomic<int> in_use;
        int gpu_to_use;

        Workspace() : in_use(0) {
            InitDeviceAllocation();
        }

        void InitDeviceAllocation()
        {
            gpu_to_use = (next_gpu_index++) % get_nb_gpu();

            CUDA_CHECK(cudaSetDevice(gpu_to_use));

            cudaStreamCreate(&stream);

            dp = (unsigned int *)malloc((GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1) * sizeof(unsigned int));

            CUDA_CHECK(cudaMallocAsync((void**)&d_dp, (GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1) * sizeof(unsigned int), stream));
            

            CUDA_CHECK(cudaMallocAsync((void**)&d_values, GENERAL_MAX_NUM_ITEMS * sizeof(unsigned int), stream));
            

            CUDA_CHECK(cudaMallocAsync((void**)&d_weights, GENERAL_MAX_NUM_ITEMS * sizeof(unsigned int), stream));
   
        }

        void initializeWorkspace(Challenge & challenge)
        {
            num_items = challenge.num_items;
            max_weight = challenge.max_weight;

            CUDA_CHECK(cudaSetDevice(gpu_to_use));

            CUDA_CHECK(cudaMemcpyAsync(d_values, challenge.values, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice, stream));

            CUDA_CHECK(cudaMemcpyAsync(d_weights, challenge.weights, num_items * sizeof(unsigned int), cudaMemcpyHostToDevice, stream));
        }



        unsigned int * retrieve_dp()
        {
            CUDA_CHECK(cudaStreamSynchronize(stream));
            CUDA_CHECK(cudaMemcpyAsync(dp, d_dp, (num_items + 1) * (max_weight + 1) * sizeof(unsigned int), cudaMemcpyDeviceToHost, stream));

            return dp;
        }

        ~Workspace()
        {
            CUDA_CHECK(cudaFreeAsync(d_dp, stream));
            CUDA_CHECK(cudaFreeAsync(d_values, stream));
            CUDA_CHECK(cudaFreeAsync(d_weights, stream));
            CUDA_CHECK(cudaStreamDestroy(stream));
            free(dp);
        }
};

static std::once_flag init_flag;

static const int nb_workspace = 128;
static std::vector<Workspace *> * workspaces = nullptr;

static void initWorkspace()
{
    std::vector<Workspace *> *l_workspaces = new std::vector<Workspace *>(nb_workspace);

    std::thread::id thread_id = std::this_thread::get_id();
    std::cout << "ThreadId = " << thread_id << " ==> Initialize workspace..." << std::endl;
    
    for (int i = 0; i < nb_workspace; i++) 
    {
        Workspace* workspace_selected = new Workspace();
        (*l_workspaces)[i] = workspace_selected;

    }
    std::cout << "ThreadId = " << thread_id << " ==> Initialize workspace OK" << std::endl;
    workspaces = l_workspaces;
}

__device__ int maxi(int a, int b) { 
	return (a > b)? a : b; 
}


__global__ void knapsackKernel(unsigned int *wt, unsigned int *val, unsigned int *dp, unsigned int W, int i, int item_index) {
        int item_weight = wt[item_index];
        int item_value = val[item_index];
        const int max_weight_plus_one = W + 1;

        for (int w = W - (blockIdx.x * blockDim.x + threadIdx.x); w >= item_weight; w -= gridDim.x * blockDim.x)
        {
            int prev_state = (i - 1) * max_weight_plus_one + w;
            int curr_state = i * max_weight_plus_one + w;
            dp[curr_state] = maxi(dp[prev_state], dp[prev_state - item_weight] + item_value);
        }
}


#include <vector>

// Comparator function for sorting by value/weight ratio
static bool compareItems(const std::pair<int, double>& a, const std::pair<int, double>& b) {
    return a.second > b.second;
}

extern "C" void solve_knapmaxxing_v2_cuda(Challenge challenge, Solution solution) {
    int n = challenge.num_items;
    int W = challenge.max_weight;

    int total_w = 0;
    int total_v = 0;
    for (int i = 0; i < n; i++)
    {
        total_w += challenge.weights[i];
        total_v += challenge.values[i];
    }
    if (total_w <= W && total_v >= challenge.min_value)
    {
        for (int i = 0; i < n; i++)
        {
            solution.items[i] = i;
        }
        return;
    }
    
    // Sort items by value/weight ratio (descending order)
    std::vector<std::pair<int, double>> sorted_items(n);
    for (int i = 0; i < n; ++i) {
        sorted_items[i] = std::make_pair(i, (double)challenge.values[i] / challenge.weights[i]);
    }
    std::sort(sorted_items.begin(), sorted_items.end(), compareItems);
    
  
    // printf("min value %d | max weight %d\n", n, W);
    // printf("input values | ");
    // for (int i = 0; i < n; i++)
    //     printf("%d ", val[i]);
    // printf("|\n");
    // printf("input weight | ");
    // for (int i = 0; i < n; i++)
    //     printf("%d ", wt[i]);
    // printf("|\n");

    // Calculate upper bound
    int upper_bound = 0;
    int remaining_weight = W;
    for (const auto& [item_index, ratio] : sorted_items) {
        int item_weight = challenge.weights[item_index];
        int item_value = challenge.values[item_index];

        if (item_weight <= remaining_weight) {
            upper_bound += item_value;
            remaining_weight -= item_weight;
        } else {
            upper_bound += (int)std::floor(ratio * remaining_weight);
            break;
        }
    }

    if (upper_bound < challenge.min_value) {
        solution.items[0] = -2; 
        return;
    }

    std::thread::id thread_id = std::this_thread::get_id();
    std::call_once(init_flag, initWorkspace);
    //std::cout << "ThreadId = " << thread_id << " ==> choosing workspace... " <<  std::endl;
    // while (workspaces == nullptr)
    // {
    //     std::call_once(init_flag, initWorkspace);
    //     std::this_thread::sleep_for(std::chrono::seconds(2));
    // }

    int workspace_id = -1; 
    Workspace *workspace_ptr = nullptr;
    while (workspace_ptr == nullptr)
    {
        int expected = 0;
        workspace_id = (next_workspace_index++) % nb_workspace;
        
        if ((*workspaces)[workspace_id]->in_use.compare_exchange_strong(expected, 1))
        {
            workspace_ptr = (*workspaces)[workspace_id];
            break;
        }
    }
   
    Workspace &workspace = *workspace_ptr;

    //std::cout << "ThreadId = " << thread_id << " ==> Choose Workspace Id = " << workspace_id << " GPU ID = " << workspace.gpu_to_use <<  std::endl;
    
    workspace.initializeWorkspace(challenge);

    
    // Dynamic Programming (DP)
    const int max_weight_plus_one = challenge.max_weight + 1;
    const int num_states = (challenge.num_items + 1) * max_weight_plus_one;
    for (int i = 1; i <= challenge.num_items; ++i) {
       const auto& [item_index, _] = sorted_items[i - 1];

       knapsackKernel<<<4, 256, 0, workspace.stream>>>(workspace.d_weights, workspace.d_values, workspace.d_dp, workspace.max_weight, i, item_index); 
    }
    unsigned int *dp = workspace.retrieve_dp();
    
    if (dp[challenge.max_weight] < challenge.min_value) {
        solution.items[0] = -2;
        workspace.in_use = 0;
        return ;
    }

    
    // 5. Récupération de la solution
    unsigned int i = challenge.num_items;
    int w = challenge.max_weight;
    int total_value = 0;
    int solution_index = 0; 
    {
        //std::lock_guard<std::mutex> lock(lock_check);

        while (i > 0 && total_value < challenge.min_value) {
            unsigned int prev_state = (i - 1) * (challenge.max_weight + 1) + w;
            unsigned int curr_state = i * (challenge.max_weight + 1) + w;
            const auto& [item_index, _] = sorted_items[i - 1];
            unsigned int item_weight = challenge.weights[item_index];
            unsigned int item_value = challenge.values[item_index];
            //printf("ThreadId = %lu ==> i = %d w = %d   item_weight %d\n", thread_id, i, w, item_weight);
            if (dp[curr_state] != dp[prev_state]) {
                solution.items[solution_index++] = item_index;
                w -= item_weight;
                
                if (w <= 0)  {
                    //printf("Break that hell %d\n", w);
                    break;
                }

                total_value += item_value;
            }
            
            i--;
        }
    }
    workspace.in_use = 0;
    if (solution_index > 0)
    {
        std::cout << "workspace ID :" << workspace_id << "GPU : " << workspace.gpu_to_use << " found potential solution" << std::endl;
    }
    else 
    {
        std::cout << "workspace ID :" << workspace_id << "GPU : " << workspace.gpu_to_use << " no solution" << std::endl;
    }
    
}
