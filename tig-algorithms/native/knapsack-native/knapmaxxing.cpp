#include <vector>
#include <stdio.h>
#include <iostream>
#include <mutex>
#include <algorithm>
#include <cstring>
#include <thread>
#include <atomic>
#include <cmath>

#include "utils.h"

static std::mutex lock_check;
//std::lock_guard<std::mutex> lock(lock_check); \

#define GENERAL_MAX_WEIGHT 10000
#define GENERAL_MAX_NUM_ITEMS 150

// Global variable to track the next GPU to assign
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
        int           num_items;
        int           max_weight;
        std::atomic<int> in_use;

        Workspace() : in_use(0) {
            InitDeviceAllocation();
        }

        void InitDeviceAllocation()
        {
            dp = (unsigned int *)malloc((GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1) * sizeof(unsigned int));
        }

        unsigned int * retrieve_dp()
        {
            std::memset(dp, 0, (GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1) * sizeof(unsigned int));
            return dp;
        }

        void initializeWorkspace(Challenge & challenge)
        {
            num_items = challenge.num_items;
            max_weight = challenge.max_weight;

        }



        ~Workspace()
        {
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


#include <vector>

// Comparator function for sorting by value/weight ratio
static bool compareItems(const std::pair<int, double>& a, const std::pair<int, double>& b) {
    return a.second > b.second;
}

#ifndef __GPU__
extern "C" void solve_knapmaxxing_v2_cuda(Challenge challenge, Solution solution) {
    int n = challenge.num_items;
    int W = challenge.max_weight;

    // int total_w = 0;
    // int total_v = 0;
    // for (int i = 0; i < n; i++)
    // {
    //     total_w += challenge.weights[i];
    //     total_v += challenge.values[i];
    // }
    // if (total_w <= W && total_v >= challenge.min_value)
    // {
    //     for (int i = 0; i < n; i++)
    //     {
    //         solution.items[i] = i;
    //     }
    //     return;
    // }
    
    // Sort items by value/weight ratio (descending order)
    std::vector<std::pair<int, double>> sorted_items(n);
    for (int i = 0; i < n; ++i) {
        sorted_items[i] = std::make_pair(i, (double)challenge.values[i] / challenge.weights[i]);
    }
    std::sort(sorted_items.begin(), sorted_items.end(), compareItems);
    
    // std::cout << "sorted_items: [";
    // for (int i = 0; i < n; i++)
    // {
    //     std::cout << sorted_items[i].first << " "; 
    // }
    // std::cout << std::endl;
    
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

    //std::cout << "Final upper bound : " << upper_bound << " Min value " << challenge.min_value << std::endl;
    
    if (upper_bound < challenge.min_value) {
        solution.items[0] = -2; 
        
        return;
    }

    std::thread::id thread_id = std::this_thread::get_id();
    std::call_once(init_flag, initWorkspace);

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

    std::cout << "ThreadId = " << thread_id << " ==> Choose Workspace Id = " << workspace_id << std::endl;
    
    workspace.initializeWorkspace(challenge);

    
    // Dynamic Programming (DP)
    const int max_weight = challenge.max_weight;
    const int max_weight_plus_one = challenge.max_weight + 1;
    const int num_states = (challenge.num_items + 1) * max_weight_plus_one;
    const int min_value = challenge.min_value;


    // Dynamic Programming (DP)
    unsigned int *dp = workspace.retrieve_dp();
    

    for (int i = 1; i <= challenge.num_items; ++i) {
        int item_index = sorted_items[i - 1].first;
        const int i_minus_one_times_max_weight_plus_one = (i - 1) * max_weight_plus_one;
        const int i_times_max_weight_plus_one = i * max_weight_plus_one;

        const int item_weight = challenge.weights[item_index];
        const int item_value = challenge.values[item_index];

        for (int w = max_weight; w >= item_weight; --w ) {
            int prev_state = i_minus_one_times_max_weight_plus_one + w;
            int curr_state = i_times_max_weight_plus_one + w;
            
            dp[curr_state] = std::max(dp[prev_state], dp[prev_state - item_weight] + item_value);
            // printf("w=%d dp[cur=%d]: %d, dp[prev=%d]: %d, dp[prev2=%d]: %d\n",
            // w, curr_state, dp[curr_state], prev_state, dp[prev_state], prev_state - item_weight, dp[prev_state - item_weight] + item_value);
            if (curr_state >= (GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1))
            {
                printf("Issue solution index out of bound %d / %d\n", curr_state, (GENERAL_MAX_NUM_ITEMS + 1) * (GENERAL_MAX_WEIGHT + 1));
            }
        }
    }
    
     // Dynamic Programming
    // for (int i = 1; i <= n; i++) {
    //     item_index = sorted_items[i - 1].first;
    //     i_minus_one_times_max_weight_plus_one = (i - 1) * max_weight_plus_one;
    //     i_times_max_weight_plus_one = i * max_weight_plus_one;

    //     int item_weight = challenge.weights[item_index];
    //     int item_value = challenge.values[item_index];

    //     for (int w = max_weight; w >= item_weight; --w) {
    //         int prev_state = i_minus_one_times_max_weight_plus_one + w;
    //         int curr_state = i_times_max_weight_plus_one + w;
            
    //         dp[curr_state] = std::max(dp[prev_state], dp[prev_state - item_weight] + item_value);

    //         printf("w=%d dp[cur=%d]: %d, dp[prev=%d]: %d, dp[prev2=%d]: %d\n",
    //         w, i * max_weight_plus_one + w, dp[curr_state], curr_state, dp[prev_state], prev_state - item_weight, dp[prev_state - item_weight] + item_value);
    //     }

    //     // // Vérification de terminaison anticipée
    //     // if (dp[i_times_max_weight_plus_one + max_weight] >= min_value) {
    //     //     printf("Break down dp[max_weight] = %d\n", dp[i_times_max_weight_plus_one + max_weight]);
    //     //     break;
    //     // }
    // }

    
    // if (dp[max_weight] < min_value) {
    //     solution.items[0] = -2;
    //     workspace.in_use = 0;
    //     return ;
    // }

    // Reconstruction de la solution
    int w = max_weight;
    int solution_index = 0;
    int total_value = 0;
    //std::cout << "Final items: [";
    for(int i = n; i > 0; i--)
    {
        int item_index = sorted_items[i - 1].first;
        int prev_state = (i - 1) * max_weight_plus_one + w;
        int curr_state = i * max_weight_plus_one + w;
        // if (w == 0 || solution_index == n) {
        //     break;
        // }
        int item_weight = challenge.weights[item_index];
        if (w >= item_weight && dp[curr_state] != dp[prev_state]) {
            if (solution_index >= challenge.num_items)
            {
                printf("Issue solution index out of bound %d / %d\n", solution_index, n);
            }

            solution.items[solution_index++] = item_index;
            //std::cout << " " << item_index << ", ";
            total_value += challenge.values[item_index];
            w -= item_weight;
        }
    }
    //std::cout << "]" << std::endl;

    workspace.in_use = 0;
    if (solution_index > 0)
    {
        if (total_value >= min_value)
            std::cout << "workspace ID :" << workspace_id << " With Values " << total_value << " vs " << challenge.min_value << " Solution found !!" << std::endl;
        else
            std::cout << "workspace ID :" << workspace_id << " With Values " << total_value << " vs " << challenge.min_value << " Not enough for solution" << std::endl;

    }
    else 
    {
        std::cout << "workspace ID :" << workspace_id << " no solution" << std::endl;
    }
    
}
#endif // __GPU__