#include "vector_search.h"
#include "RngArray.h"
#include <random>
#include <algorithm>
#include <cmath>
#include <stdexcept>
#include <numeric>
#include <vector>

extern "C"
{
    // Fonction pour calculer la distance euclidienne entre deux vecteurs
    static float euclidean_distance(const float *a, const float *b)
    {
        float sum = 0.0f;
        for (size_t i = 0; i < 250; ++i)
        {
            float diff = a[i] - b[i];
            sum += diff * diff;
        }
        return std::sqrt(sum);
    }

    // Génération d'une instance de VSOChallenge
    VSOChallenge *generate_instance_vs(const uint64_t * seeds, const VSODifficulty *difficulty)
    {
        RngArrayNative* rng = rng_array_native_new(seeds);

        //RngArray rngs(seeds);
        // // Génération de RNGs à partir des seeds
        // std::uniform_real_distribution<float> uniform(0.0f, 1.0f);

        // Génération de la base de données vectorielle
        float **vector_database = new float *[100000];
        for (size_t i = 0; i < 100000; ++i)
        {
            vector_database[i] = new float[250];
            for (size_t j = 0; j < 250; ++j)
            {
                vector_database[i][j] = rng_array_native_sample_uniform32(rng, 0.0, 1.0);
            }
        }

        // Génération des vecteurs de requête
        float **query_vectors = new float *[difficulty->num_queries];
        for (size_t i = 0; i < difficulty->num_queries; ++i)
        {
            query_vectors[i] = new float[250];
            for (size_t j = 0; j < 250; ++j)
            {
                query_vectors[i][j] = rng_array_native_sample_uniform32(rng, 0.0, 1.0);
            }
        }

        // Calcul de la distance maximale
        float max_distance = 6.0f - static_cast<float>(difficulty->better_than_baseline) / 1000.0f;

        // Création du challenge
        VSOChallenge *challenge = new VSOChallenge;
        std::copy(seeds, seeds + 8, ((uint64_t*)challenge->seeds));
        challenge->difficulty.better_than_baseline = difficulty->better_than_baseline;
        challenge->difficulty.num_queries = difficulty->num_queries;
        challenge->vector_database = vector_database;
        challenge->vector_database_size = 100000; // Taille du vector_database
        challenge->query_vectors = query_vectors;
        challenge->query_vectors_size = difficulty->num_queries;
        challenge->max_distance = max_distance;

        return challenge;
    }

    // Vérification de la solution
    unsigned int verify_solution_vs(const VSOChallenge *challenge, const VSOSolution *solution)
    {
        if (solution->len != challenge->difficulty.num_queries)
        {
            return 1; // Nombre d'indexes incorrect
        }

        std::vector<float> distances;
        for (size_t i = 0; i < challenge->query_vectors_size; ++i)
        {
            size_t search_index = solution->indexes[i];
            if (search_index >= challenge->vector_database_size)
            {
                return 2; // Index hors bornes
            }

            // Calcul de la distance entre le vecteur de requête et le vecteur de base de données
            float dist = euclidean_distance(challenge->query_vectors[i], challenge->vector_database[search_index]);
            distances.push_back(dist);
        }

        // Calcul de la distance moyenne
        float avg_distance = std::accumulate(distances.begin(), distances.end(), 0.0f) / distances.size();
        if (avg_distance > challenge->max_distance)
        {
            return 3; // Distance moyenne trop grande
        }

        return 0; // Solution valide
    }

    // Fonction pour libérer la mémoire allouée pour un VSOChallenge
    void free_vso_challenge(VSOChallenge *challenge)
    {
        if (!challenge)
        {
            return;
        }

        // Libération de la mémoire pour vector_database
        for (size_t i = 0; i < challenge->vector_database_size; ++i)
        {
            delete[] challenge->vector_database[i];
        }
        delete[] challenge->vector_database;

        // Libération de la mémoire pour query_vectors
        for (size_t i = 0; i < challenge->query_vectors_size; ++i)
        {
            delete[] challenge->query_vectors[i];
        }
        delete[] challenge->query_vectors;

        // Libération de l'objet challenge lui-même
        delete challenge;
    }
}
