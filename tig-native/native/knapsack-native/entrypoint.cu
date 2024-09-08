#include <iostream>
#include <vector>
#include <algorithm>
#include <numeric>
#include <random>
#include <set>
#include <cmath>
#include <iostream>
#include <vector>
#include <algorithm>
#include <numeric>
#include <random>
#include <set>
#include <cmath>
#include <cstring>
#include <cassert>
#include "dp_cuda.h"


#include <atomic>

static std::atomic<unsigned int> next_worker(0);

extern "C" void solve_knapmaxxing_v1_cuda(Challenge challenge, Solution solution) {

    int next_worker_to_use = next_worker++ % (128 + 64);
    
    if (next_worker_to_use < 128) // Test to jump faster in the random data
        return;

    if (next_worker_to_use >= 128 + 32)
    {
        solve_knapmaxxing_cpp(challenge, solution);
        return;
    }

    weight_t capacity = challenge.max_weight;
    int num_items = challenge.num_items;
    // Create a vector of indices
    std::vector<unsigned int> indices(num_items);
    for (int i = 0; i < num_items; ++i) {
        indices[i] = i;
    }

    // // Sort indices based on the ratio of value to weight
    // std::sort(indices.begin(), indices.end(), [&](int a, int b) {
    //     return (float)challenge.values[a] / challenge.weights[a] > (float)challenge.values[b] / challenge.weights[b];
    // });

    // // Apply sorted indices to weights and values
    // std::vector<unsigned int> sorted_weights(num_items);
    // std::vector<unsigned int> sorted_values(num_items);
    // for (int i = 0; i < num_items; ++i) {
    //     sorted_weights[i] = challenge.weights[indices[i]];
    //     sorted_values[i] = challenge.values[indices[i]];
    // }


    // weight_t* weights = sorted_weights.data();
    // value_t* values = sorted_values.data();

    const weight_t* weights = challenge.weights;
    const value_t* values = challenge.values;

    // std::cout << "Max weight : " << challenge.max_weight << std::endl;
    // std::cout << "Min value  : " << challenge.min_value << std::endl;

    // std::cout << "Values : ";
    // for (size_t i = 0; i < num_items; ++i) {
    //     std::cout << challenge.values[i] << " ";
    // }
    // std::cout << std::endl;
    // std::cout << "Weight : ";
    // for (size_t i = 0; i < num_items; ++i) {
    //     std::cout << challenge.weights[i] << " ";
    // }
    // std::cout << std::endl;

    uint32_t* taken_indices = new uint32_t[num_items];
    memset(taken_indices, -1, num_items * sizeof(uint32_t));

    value_t best = gpu_knapsack(capacity, weights, values, num_items, taken_indices);

    // // Affichage pour vérifier les résultats
    // std::cout << "Best value: " << best << std::endl;
    // std::cout << "Taken indices: ";
    // for (size_t i = 0; i < num_items; ++i) {
    //     if (taken_indices[i] != -1) {
    //         std::cout << indices[taken_indices[i]] << " ";
    //     }
    // }
    // std::cout << std::endl;


    std::vector<bool> taken(num_items, false);
    // Vérification des indices pris
    std::vector<size_t> solution_items;
    int count = 0;
    for (size_t i = 0; i < num_items; ++i) {
        if (taken_indices[i] != -1 && !taken[indices[taken_indices[i]]]) {
            solution.items[count++] = indices[taken_indices[i]];
            taken[indices[taken_indices[i]]] = true;
        }
    }

    if (count < num_items)
    {
        memset(solution.items + count, -1, (num_items - count) * sizeof(int32_t));
    }

    delete[] taken_indices;
}
