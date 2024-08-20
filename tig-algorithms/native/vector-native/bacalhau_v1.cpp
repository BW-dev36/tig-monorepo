#include <cmath>
#include <cstddef>
#include <cfloat> // For FLT_MAX

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
    };

    struct VSSolution
    {
        size_t *indexes;
        size_t len;
    };

    inline float l2_norm(const float *vec, size_t len)
    {
        float sum = 0.0f;
        for (size_t i = 0; i < len; ++i)
        {
            sum += vec[i] * vec[i];
        }
        return std::sqrt(sum);
    }

    inline float euclidean_distance_with_precomputed_norm(
        float a_norm_sq,
        float b_norm_sq,
        float ab_dot_product)
    {
        return std::sqrt(a_norm_sq + b_norm_sq - 2.0f * ab_dot_product);
    }

    void solve_bacalhau_v1_cpp(const VSChallenge *challenge, VSSolution *solution)
    {
        const float *vector_database = challenge->vector_database;
        const size_t *vector_sizes = challenge->vector_sizes;
        size_t num_vectors = challenge->num_vectors;

        const float *query_vectors = challenge->query_vectors;
        const size_t *query_sizes = challenge->query_sizes;
        size_t num_queries = challenge->num_queries;

        float max_distance = challenge->max_distance;

        solution->len = 0;

        float *vector_norms_sq = new float[num_vectors];
        float sum_norms_sq = 0.0f;
        float sum_squares = 0.0f;

        size_t vector_offset = 0;
        for (size_t i = 0; i < num_vectors; ++i)
        {
            size_t len = vector_sizes[i];
            float norm_sq = 0.0f;
            for (size_t j = 0; j < len; ++j)
            {
                norm_sq += vector_database[vector_offset + j] * vector_database[vector_offset + j];
            }
            sum_norms_sq += std::sqrt(norm_sq);
            sum_squares += norm_sq;
            vector_norms_sq[i] = norm_sq;
            vector_offset += len;
        }

        float vector_norms_len = static_cast<float>(num_vectors);
        float std_dev = std::sqrt((sum_squares / vector_norms_len) - std::pow(sum_norms_sq / vector_norms_len, 2));
        float norm_threshold = 2.0f * std_dev;

        size_t query_offset = 0;
        for (size_t i = 0; i < num_queries; ++i)
        {
            size_t query_len = query_sizes[i];
            float query_norm_sq = 0.0f;
            for (size_t j = 0; j < query_len; ++j)
            {
                query_norm_sq += query_vectors[query_offset + j] * query_vectors[query_offset + j];
            }

            size_t closest_index = num_vectors; // Initialiser à un index invalide
            float closest_distance = FLT_MAX;

            vector_offset = 0;
            for (size_t idx = 0; idx < num_vectors; ++idx)
            {
                size_t vector_len = vector_sizes[idx];
                float vector_norm_sq = vector_norms_sq[idx];

                if (std::abs(std::sqrt(vector_norm_sq) - std::sqrt(query_norm_sq)) > norm_threshold)
                {
                    vector_offset += vector_len;
                    continue;
                }

                float ab_dot_product = 0.0f;
                for (size_t j = 0; j < query_len; ++j)
                {
                    ab_dot_product += query_vectors[query_offset + j] * vector_database[vector_offset + j];
                }

                float distance = euclidean_distance_with_precomputed_norm(
                    query_norm_sq,
                    vector_norm_sq,
                    ab_dot_product);

                if (distance <= max_distance)
                {
                    closest_index = idx;
                    break; // Sortie anticipée
                }
                else if (distance < closest_distance)
                {
                    closest_index = idx;
                    closest_distance = distance;
                }

                vector_offset += vector_len;
            }

            if (closest_index != num_vectors)
            {
                solution->indexes[solution->len++] = closest_index;
            }
            else
            {
                solution->len = 0; // Pas de solution trouvée
                delete[] vector_norms_sq;
                return;
            }

            query_offset += query_len;
        }

        delete[] vector_norms_sq;
    }
}
