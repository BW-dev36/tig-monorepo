#include "knapsack.h"
#include <random>
#include <algorithm>
#include <cmath>
#include <set>
#include <stdexcept>

extern "C" {

KChallenge* generate_instance(const uint64_t * seeds, const KDifficulty& difficulty) {
    // Génération de RNGs à partir des seeds
    std::mt19937 rng(seeds[0]);  // Mersenne Twister RNG

    // Génération des poids et valeurs
    std::vector<uint32_t> weights(difficulty.num_items);
    std::vector<uint32_t> values(difficulty.num_items);
    
    for (size_t i = 0; i < difficulty.num_items; ++i) {
        weights[i] = rng() % 49 + 1;  // Génère entre 1 et 50
        values[i] = rng() % 49 + 1;   // Génère entre 1 et 50
    }

    // Calcul du poids maximum
    uint32_t max_weight = std::accumulate(weights.begin(), weights.end(), 0u) / 2;

    // Algorithme glouton de base (tri par ratio valeur/poids)
    std::vector<size_t> sorted_value_to_weight_ratio(difficulty.num_items);
    std::iota(sorted_value_to_weight_ratio.begin(), sorted_value_to_weight_ratio.end(), 0);

    std::sort(sorted_value_to_weight_ratio.begin(), sorted_value_to_weight_ratio.end(),
              [&](size_t a, size_t b) {
                  double ratio_a = static_cast<double>(values[a]) / weights[a];
                  double ratio_b = static_cast<double>(values[b]) / weights[b];
                  return ratio_b < ratio_a;
              });

    // Calcul du minimum de valeur
    uint32_t total_weight = 0;
    uint32_t min_value = 0;

    for (size_t i : sorted_value_to_weight_ratio) {
        if (total_weight + weights[i] > max_weight) {
            continue;
        }
        min_value += values[i];
        total_weight += weights[i];
    }

    min_value = static_cast<uint32_t>(std::round(min_value * (1.0 + difficulty.better_than_baseline / 1000.0)));

    // Création du challenge
    KChallenge* challenge = new KChallenge;
    challenge->seeds = seeds;
    challenge->difficulty = difficulty;
    challenge->weights = std::move(weights);
    challenge->values = std::move(values);
    challenge->max_weight = max_weight;
    challenge->min_value = min_value;

    return challenge;
}

int verify_solution_knap(const KChallenge* challenge, const KSolution* solution) {
    std::set<size_t> selected_items(solution->items.begin(), solution->items.end());

    if (selected_items.size() != solution->items.size()) {
        return -1;  // Doublon dans les items sélectionnés
    }

    // Vérification des bornes des items
    for (size_t item : selected_items) {
        if (item >= challenge->weights.size()) {
            return -2;  // Item hors bornes
        }
    }

    // Calcul du poids total
    uint32_t total_weight = 0;
    for (size_t item : selected_items) {
        total_weight += challenge->weights[item];
    }

    if (total_weight > challenge->max_weight) {
        return -3;  // Poids total dépassé
    }

    // Calcul de la valeur totale
    uint32_t total_value = 0;
    for (size_t item : selected_items) {
        total_value += challenge->values[item];
    }

    if (total_value < challenge->min_value) {
        return -4;  // Valeur minimale non atteinte
    }

    return 0;  // Solution valide
}

}
