#ifndef KNAPSACK_H
#define KNAPSACK_H
#include <cstdint>
#include <stdint.h>
#include <stddef.h>
#include <vector>
#include <array>

extern "C" {

// Structure Difficulty
struct KDifficulty {
    size_t num_items;
    uint32_t better_than_baseline;
};

// Structure Solution
struct KSolution {
    std::vector<size_t> items;
};

// Structure Challenge
struct KChallenge {
    const uint64_t* seeds;
    KDifficulty difficulty;
    std::vector<uint32_t> weights;
    std::vector<uint32_t> values;
    uint32_t max_weight;
    uint32_t min_value;
};

// Fonctions externes
KChallenge* generate_instance(const uint64_t* seeds, const KDifficulty& difficulty);
int verify_solution(const KChallenge* challenge, const KSolution* solution);

}

#endif // KNAPSACK_H
