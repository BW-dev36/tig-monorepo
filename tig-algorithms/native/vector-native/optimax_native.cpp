#include <iostream>
#include <vector>
#include <cmath>
#include <queue>
#include <algorithm>
#include <cfloat> // For FLT_MAX
#include <chrono>

extern "C"
{

    struct VSChallenge
    {
        const float *vector_database;
        size_t vector_database_len;
        const size_t *vector_sizes;
        size_t num_vectors;

        const float *query_vectors;
        size_t query_vectors_len;
        const size_t *query_sizes;
        size_t num_queries;

        float max_distance;
        unsigned int difficulty;
    };

    struct VSSolution
    {
        size_t *indexes;
        size_t len;
    };

    struct KDNode
    {
        const float *point;
        KDNode *left;
        KDNode *right;
        size_t index;

        KDNode(const float *pt, size_t idx) : point(pt), left(nullptr), right(nullptr), index(idx) {}
    };

    inline float squared_euclidean_distance(const float *a, const float *b, size_t len)
    {
        float sum = 0.0f;
        for (size_t i = 0; i < len; ++i)
        {
            float diff = a[i] - b[i];
            sum += diff * diff;
        }
        return sum;
    }

    inline float early_stopping_distance(const float *a, const float *b, size_t len, float current_min)
    {
        float sum = 0.0f;
        size_t i = 0;

        while (i + 3 < len)
        {
            float diff0 = a[i] - b[i];
            float diff1 = a[i + 1] - b[i + 1];
            float diff2 = a[i + 2] - b[i + 2];
            float diff3 = a[i + 3] - b[i + 3];

            sum += diff0 * diff0 + diff1 * diff1 + diff2 * diff2 + diff3 * diff3;

            if (sum > current_min)
            {
                return FLT_MAX;
            }

            i += 4;
        }

        while (i < len)
        {
            float diff = a[i] - b[i];
            sum += diff * diff;

            if (sum > current_min)
            {
                return FLT_MAX;
            }

            i += 1;
        }

        return sum;
    }

    std::vector<float> calculate_mean_vector(const std::vector<const float *> &vectors, size_t num_dimensions)
    {
        std::vector<float> mean_vector(num_dimensions, 0.0f);

        for (const float *vector : vectors)
        {
            for (size_t i = 0; i < num_dimensions; ++i)
            {
                mean_vector[i] += vector[i];
            }
        }

        for (size_t i = 0; i < num_dimensions; ++i)
        {
            mean_vector[i] /= static_cast<float>(vectors.size());
        }

        return mean_vector;
    }

    std::vector<size_t> filter_relevant_vectors(const VSChallenge *challenge, const float *mean_query_vector, size_t k)
    {
        std::priority_queue<std::pair<float, size_t>> heap;

        size_t num_dimensions = 250;
        size_t offset = 0;

        for (size_t index = 0; index < challenge->num_vectors; ++index)
        {
            const float *vector = &challenge->vector_database[offset];
            offset += challenge->vector_sizes[index];

            float dist = squared_euclidean_distance(mean_query_vector, vector, num_dimensions);

            if (heap.size() < k)
            {
                heap.push({dist, index});
            }
            else if (dist < heap.top().first)
            {
                heap.pop();
                heap.push({dist, index});
            }
        }

        std::vector<size_t> result;
        while (!heap.empty())
        {
            result.push_back(heap.top().second);
            heap.pop();
        }

        return result;
    }

    KDNode *build_kd_tree(const float *points, size_t *indices, size_t *sizes, size_t start, size_t end, size_t depth, size_t num_dimensions)
    {
        if (start >= end)
            return nullptr;

        size_t axis = depth % num_dimensions;
        size_t median = (start + end) / 2;

        std::nth_element(indices + start, indices + median, indices + end, [&](size_t lhs, size_t rhs)
                         { return points[lhs * num_dimensions + axis] < points[rhs * num_dimensions + axis]; });

        KDNode *node = new KDNode(points + indices[median] * num_dimensions, indices[median]);

        node->left = build_kd_tree(points, indices, sizes, start, median, depth + 1, num_dimensions);
        node->right = build_kd_tree(points, indices, sizes, median + 1, end, depth + 1, num_dimensions);

        return node;
    }

    void nearest_neighbor_search(KDNode *node, const float *target, size_t len, size_t depth, float &best_dist, size_t &best_index)
    {
        if (!node)
            return;

        size_t axis = depth % len;
        float dist = early_stopping_distance(node->point, target, len, best_dist);

        if (dist < best_dist)
        {
            best_dist = dist;
            best_index = node->index;
        }

        float diff = target[axis] - node->point[axis];
        float sqr_diff = diff * diff;

        KDNode *nearer_node = diff < 0.0f ? node->left : node->right;
        KDNode *farther_node = diff < 0.0f ? node->right : node->left;

        nearest_neighbor_search(nearer_node, target, len, depth + 1, best_dist, best_index);

        if (sqr_diff < best_dist)
        {
            nearest_neighbor_search(farther_node, target, len, depth + 1, best_dist, best_index);
        }
    }

    void solve_optimax_cpp(const VSChallenge *challenge, VSSolution *solution)
    {
        auto start_total = std::chrono::high_resolution_clock::now();

        size_t num_dimensions = 250;
        std::vector<const float *> query_refs(challenge->num_queries);

        size_t query_offset = 0;
        for (size_t i = 0; i < challenge->num_queries; ++i)
        {
            query_refs[i] = &challenge->query_vectors[query_offset];
            query_offset += challenge->query_sizes[i];
        }

        //auto start_mean_calc = std::chrono::high_resolution_clock::now();
        std::vector<float> mean_query_vector = calculate_mean_vector(query_refs, num_dimensions);
        // auto end_mean_calc = std::chrono::high_resolution_clock::now();
        // std::cout << "Time taken for mean vector calculation: "
        //           << std::chrono::duration_cast<std::chrono::milliseconds>(end_mean_calc - start_mean_calc).count()
        //           << " ms" << std::endl;

        //auto start_filtering = std::chrono::high_resolution_clock::now();
        int subset_size = 1000; // Valeur par défaut

        // Condition pour déterminer subset_size
        if (challenge->num_queries >= 10 && challenge->num_queries <= 19)
        {
            subset_size = 4200;
        }
        else if (challenge->num_queries >= 20 && challenge->num_queries <= 28)
        {
            subset_size = (challenge->difficulty <= 465) ? 3000 : 6000;
        }
        else if (challenge->num_queries >= 29 && challenge->num_queries <= 50)
        {
            if (challenge->difficulty <= 480)
            {
                subset_size = 2000;
            }
            else if (challenge->num_queries <= 45 && challenge->difficulty > 480)
            {
                subset_size = 6000;
            }
            else
            {
                subset_size = 5000;
            }
        }
        else if (challenge->num_queries >= 51 && challenge->num_queries <= 70)
        {
            subset_size = 3000;
        }
        else if (challenge->num_queries >= 71 && challenge->num_queries <= 100)
        {
            subset_size = (challenge->difficulty <= 480) ? 1500 : 2500;
        }

        std::vector<size_t> relevant_indices = filter_relevant_vectors(challenge, mean_query_vector.data(), subset_size);
        //auto end_filtering = std::chrono::high_resolution_clock::now();
        // std::cout << "Time taken for filtering relevant vectors: "
        //           << std::chrono::duration_cast<std::chrono::milliseconds>(end_filtering - start_filtering).count()
        //           << " ms" << std::endl;

        //auto start_kd_tree = std::chrono::high_resolution_clock::now();
        std::vector<size_t> subset_sizes(relevant_indices.size());
        size_t total_points = 0;

        for (size_t i = 0; i < relevant_indices.size(); ++i)
        {
            subset_sizes[i] = challenge->vector_sizes[relevant_indices[i]];
            total_points += subset_sizes[i];
        }

        std::vector<float> subset_database(total_points);
        std::vector<size_t> subset_indices(relevant_indices.size());

        size_t offset = 0;
        for (size_t i = 0; i < relevant_indices.size(); ++i)
        {
            size_t idx = relevant_indices[i];
            const float *src_ptr = &challenge->vector_database[idx * num_dimensions];
            std::copy(src_ptr, src_ptr + subset_sizes[i], &subset_database[offset]);
            subset_indices[i] = offset / num_dimensions;
            offset += subset_sizes[i];
        }

        KDNode *kd_tree = build_kd_tree(subset_database.data(), subset_indices.data(), subset_sizes.data(), 0, subset_indices.size(), 0, num_dimensions);
        // auto end_kd_tree = std::chrono::high_resolution_clock::now();
        // std::cout << "Time taken for building KD-Tree: "
        //           << std::chrono::duration_cast<std::chrono::milliseconds>(end_kd_tree - start_kd_tree).count()
        //           << " ms" << std::endl;

        //auto start_search = std::chrono::high_resolution_clock::now();
        solution->len = 0;

        for (size_t i = 0; i < challenge->num_queries; ++i)
        {
            const float *query = challenge->query_vectors + i * num_dimensions;
            float best_dist = FLT_MAX;
            size_t best_index = 0;

            nearest_neighbor_search(kd_tree, query, num_dimensions, 0, best_dist, best_index);
            solution->indexes[solution->len++] = relevant_indices[best_index];
        }
        // auto end_search = std::chrono::high_resolution_clock::now();
        // std::cout << "Time taken for nearest neighbor search: "
        //           << std::chrono::duration_cast<std::chrono::milliseconds>(end_search - start_search).count()
        //           << " ms" << std::endl;

        // Cleanup KD-Tree
        std::vector<KDNode *> stack;
        if (kd_tree)
            stack.push_back(kd_tree);

        while (!stack.empty())
        {
            KDNode *node = stack.back();
            stack.pop_back();

            if (node->left)
                stack.push_back(node->left);
            if (node->right)
                stack.push_back(node->right);

            delete node;
        }

        // auto end_total = std::chrono::high_resolution_clock::now();
        // std::cout << "Total time taken by solve_optimax_cpp: "
        //           << std::chrono::duration_cast<std::chrono::milliseconds>(end_total - start_total).count()
        //           << " ms" << std::endl;
    }

} // extern "C"
