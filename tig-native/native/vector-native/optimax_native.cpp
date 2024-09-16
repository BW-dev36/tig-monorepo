#include <vector>
#include <array>
#include <cstdint>
#include <algorithm>
#include <queue>
#include <limits>
#include <cmath>
#include "../challenges/vector_search.h"
#include <iostream>

// Fonction de distance euclidienne au carré
inline float squared_euclidean_distance(const float * a, const float * b) {
    float sum = 0.0f;
    for (size_t i = 0; i < 250; ++i) {
        float diff = a[i] - b[i];
        sum += diff * diff;
    }
    return sum;
}

// Fonction d'arrêt anticipé pour optimiser la recherche des voisins proches
inline float early_stopping_distance(const float * a, const float* b, float current_min) {
    float sum = 0.0f;
    size_t i = 0;
    while (i + 3 < 250) {
        float diff0 = a[i] - b[i];
        float diff1 = a[i + 1] - b[i + 1];
        float diff2 = a[i + 2] - b[i + 2];
        float diff3 = a[i + 3] - b[i + 3];
        sum += diff0 * diff0 + diff1 * diff1 + diff2 * diff2 + diff3 * diff3;
        if (sum > current_min) {
            return std::numeric_limits<float>::max();
        }
        i += 4;
    }
    while (i < 250) {
        float diff = a[i] - b[i];
        sum += diff * diff;
        if (sum > current_min) {
            return std::numeric_limits<float>::max();
        }
        i++;
    }
    return sum;
}

// Fonction quickselect pour optimiser le tri partiel
template <typename T, typename Compare>
void quickselect(std::vector<T>& arr, size_t left, size_t right, size_t k, Compare comp) {
    if (left >= right) {
        return;
    }

    size_t pivot_index = partition(arr, left, right, comp);
    if (k < pivot_index) {
        quickselect(arr, left, pivot_index - 1, k, comp);
    } else if (k > pivot_index) {
        quickselect(arr, pivot_index + 1, right, k, comp);
    }
}

// Partition utilisée dans quickselect
template <typename T, typename Compare>
size_t partition(std::vector<T>& arr, size_t left, size_t right, Compare comp) {
    size_t pivot_index = left + (right - left) / 2;  // Utilisation de la médiane comme pivot
    std::swap(arr[pivot_index], arr[right]);         // Déplacer le pivot à la fin

    size_t store_index = left;
    for (size_t i = left; i < right; ++i) {
        if (comp(arr[i], arr[right])) {
            std::swap(arr[i], arr[store_index]);
            store_index++;
        }
    }
    std::swap(arr[store_index], arr[right]);  // Déplacer le pivot à sa place finale
    return store_index;
}

// Fonction pour calculer le vecteur moyen
std::vector<float> calculate_mean_vector(const std::vector<float*>& vectors) {
    size_t num_vectors = vectors.size();
    const size_t num_dimensions = 250;

    std::vector<float> mean_vector(num_dimensions, 0.0f);

    for (const auto* vector : vectors) {
        for (size_t i = 0; i < num_dimensions; ++i) {
            mean_vector[i] += vector[i];
        }
    }

    for (size_t i = 0; i < num_dimensions; ++i) {
        mean_vector[i] /= num_vectors;
    }

    return mean_vector;
}

// Fonction pour filtrer les vecteurs pertinents
std::vector<std::pair<const float*, size_t>> filter_relevant_vectors(const VSOChallenge* challenge, 
    size_t k
) {
    // Obtenez le vecteur moyen des vecteurs de requête
    std::vector<float*> query_refs;
    for (size_t i = 0; i < challenge->query_vectors_size; ++i) {
        float* query = challenge->query_vectors[i];
        query_refs.push_back(query);
    }

    std::vector<float> mean_query_vector = calculate_mean_vector(query_refs);

    // Utilisez un tas pour garder les k meilleurs vecteurs
    using PairType = std::pair<float, size_t>;
    std::priority_queue<PairType> heap;

    for (size_t index = 0; index < challenge->vector_database_size; ++index) {
        float dist = squared_euclidean_distance(mean_query_vector.data(), challenge->vector_database[index]);
        if (heap.size() < k) {
            heap.emplace(dist, index);
        } else if (dist < heap.top().first) {
            heap.pop();
            heap.emplace(dist, index);
        }
    }

    // Construit un tableau avec les résultats filtrés
    std::vector<std::pair<const float *, size_t>> relevant_vectors;
    while (!heap.empty()) {
        auto [dist, index] = heap.top();
        heap.pop();
        relevant_vectors.emplace_back(challenge->vector_database[index], index);
    }

    return relevant_vectors;
}

// Construction de l'arbre KD
struct KDNode {
    const float* point;
    KDNode* left = nullptr;
    KDNode* right = nullptr;
    size_t index;

    KDNode(const float* point, size_t index) : point(point), index(index) {}
};

KDNode* build_kd_tree(std::vector<std::pair<const float*, size_t>>& points, size_t depth = 0) {
    if (points.empty()) {
        return nullptr;
    }

    size_t axis = depth % 250;
    size_t median = points.size() / 2;
    // Appel correct de quickselect avec les bornes gauche et droite
    quickselect(points, 0, points.size() - 1, median, [&](const auto& a, const auto& b) {
        return a.first[axis] < (b.first)[axis];
    });

    KDNode* node = new KDNode(points[median].first, points[median].second);
    std::vector<std::pair<const float*, size_t>> left(points.begin(), points.begin() + median);
    std::vector<std::pair<const float*, size_t>> right(points.begin() + median + 1, points.end());

    node->left = build_kd_tree(left, depth + 1);
    node->right = build_kd_tree(right, depth + 1);

    return node;
}

// Recherche du voisin le plus proche dans l'arbre KD
void nearest_neighbor_search(KDNode* root, const float* target, std::pair<float, size_t>& best, size_t depth = 0) {
    if (!root) {
        return;
    }

    float dist = early_stopping_distance(root->point, target, best.first);
    if (dist < best.first) {
        best.first = dist;
        best.second = root->index;
    }

    size_t axis = depth % 250;
    float diff = target[axis] - root->point[axis];

    KDNode* near_branch = diff < 0 ? root->left : root->right;
    KDNode* far_branch = diff < 0 ? root->right : root->left;

    nearest_neighbor_search(near_branch, target, best, depth + 1);
    if (diff * diff < best.first) {
        nearest_neighbor_search(far_branch, target, best, depth + 1);
    }
}

// Fonction solve_challenge
void solve_optimax_cpp(const VSOChallenge* challenge, VSOSolution* solution) {
    size_t query_count = challenge->difficulty.num_queries;
    //auto start_filtering = std::chrono::high_resolution_clock::now();
        int subset_size = 1000; // Valeur par défaut

        // Condition pour déterminer subset_size
        if (challenge->difficulty.num_queries >= 10 && challenge->difficulty.num_queries <= 19)
        {
            subset_size = 4200;
        }
        else if (challenge->difficulty.num_queries >= 20 && challenge->difficulty.num_queries <= 28)
        {
            subset_size = (challenge->difficulty.better_than_baseline <= 465) ? 3000 : 6000;
        }
        else if (challenge->difficulty.num_queries >= 29 && challenge->difficulty.num_queries <= 50)
        {
            if (challenge->difficulty.better_than_baseline <= 480)
            {
                subset_size = 2000;
            }
            else if (challenge->difficulty.num_queries <= 45 && challenge->difficulty.better_than_baseline > 480)
            {
                subset_size = 6000;
            }
            else
            {
                subset_size = 5000;
            }
        }
        else if (challenge->difficulty.num_queries >= 51 && challenge->difficulty.num_queries <= 70)
        {
            subset_size = 3000;
        }
        else if (challenge->difficulty.num_queries >= 71 && challenge->difficulty.num_queries <= 100)
        {
            subset_size = (challenge->difficulty.better_than_baseline <= 480) ? 1500 : 2500;
        }


    std::vector<std::pair<const float*, size_t>> relevant_vectors =
        filter_relevant_vectors(challenge, subset_size);

    KDNode* kd_tree = build_kd_tree(relevant_vectors);

    for (size_t i = 0; i < challenge->query_vectors_size; ++i) {
        float* query = challenge->query_vectors[i];
        std::pair<float, size_t> best(std::numeric_limits<float>::max(), 0);
        nearest_neighbor_search(kd_tree, query, best);
        solution->indexes[solution->len++] = best.second;
    }
}

unsigned int solve_optimax_cpp_full(const uint64_t * seeds, const VSODifficulty *difficulty) 
{
    auto *workspace = generate_instance_vs(seeds, difficulty);
    
    solve_optimax_cpp(workspace->challenge, workspace->solution);
    int res = verify_solution_vs(workspace->challenge, workspace->solution);
    workspace->in_use = 0;
    return res;
}