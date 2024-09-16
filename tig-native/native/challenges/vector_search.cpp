#include "vector_search.h"
#include "RngArray.h"
#include <random>
#include <algorithm>
#include <cmath>
#include <stdexcept>
#include <numeric>
#include <vector>

#include <stdio.h>
#include <iostream>
#include <mutex>
#include <atomic>
#include <cstring>
#include <thread>

static std::mutex lock_check;
//std::lock_guard<std::mutex> lock(lock_check); \

#define GENERAL_MAX_WEIGHT 10000
#define GENERAL_MAX_NUM_ITEMS 150

// Global variable to track the next GPU to assign
static std::atomic<unsigned int> next_Workspace_vs_index(0);

extern "C"
{
    Workspace_vs::Workspace_vs() : in_use(0)
    {
        InitDeviceAllocation();
    }

    void Workspace_vs::InitDeviceAllocation()
    {
        challenge = new VSOChallenge;
        solution = new VSOSolution;
        solution->indexes = new size_t[1000];
        solution->len = 0;

        vector_database = new float *[100000];
        query_vectors = new float *[1000];
        for (size_t i = 0; i < 100000; ++i)
        {
            vector_database[i] = new float[250];

            if (i < 1000)
                query_vectors[i] = new float[250];
        }
    }

    Workspace_vs::~Workspace_vs()
    {
        delete solution;
        delete challenge;
        for (int i = 0; i < 100000; i++)
        {
            delete vector_database[i];
            if (i < 1000)
                delete query_vectors[i];
        }
        delete vector_database;
        delete query_vectors;
    }

    static std::once_flag init_flag;

    static const int nb_Workspace_vs = 128;
    static std::vector<Workspace_vs *> *workspaces_vs = nullptr;

    static void initWorkspace_vs()
    {
        std::vector<Workspace_vs *> *l_Workspace_vs = new std::vector<Workspace_vs *>(nb_Workspace_vs);

        std::thread::id thread_id = std::this_thread::get_id();
        std::cout << "ThreadId = " << thread_id << " ==> Initialize Workspace_vs..." << std::endl;

        for (int i = 0; i < nb_Workspace_vs; i++)
        {
            Workspace_vs *Workspace_vs_selected = new Workspace_vs();
            (*l_Workspace_vs)[i] = Workspace_vs_selected;
        }
        std::cout << "ThreadId = " << thread_id << " ==> Initialize Workspace_vs OK" << std::endl;
        workspaces_vs = l_Workspace_vs;
    }

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

#include <immintrin.h> // Pour AVX-512
#include <stddef.h>    // Pour size_t

    // Suppose que rng_array_native_sample_uniform32 génère un seul float aléatoire
    void generate_random_floats_avx512(float *result, size_t count, RngArrayNative *rng)
    {
        size_t i = 0;

        // Traite par blocs de 16 éléments
        for (; i + 16 <= count; i += 16)
        {
            __m512 random_values = _mm512_set_ps(
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0),
                rng_array_native_sample_uniform32(rng, 0.0, 1.0));
            _mm512_storeu_ps(&result[i], random_values); // Stocke les 16 floats
        }

        // Gère les éléments restants (moins de 16)
        if (i < count)
        {
            for (size_t j = 0; j < count - i; ++j)
            {
                result[i + j] = rng_array_native_sample_uniform32(rng, 0.0, 1.0); // Copie les valeurs dans le résultat final
            }
        }
    }

    // Génération de la base de données vectorielle avec AVX-512
    void generate_vector_database_avx512(float **vector_database, size_t database_size, RngArrayNative *rng)
    {
        for (size_t i = 0; i < database_size; ++i)
        {
            generate_random_floats_avx512(vector_database[i], 250, rng); // Traite 250 floats par ligne
        }
    }

    // Génération des vecteurs de requête avec AVX-512
    void generate_query_vectors_avx512(float **query_vectors, size_t num_queries, RngArrayNative *rng)
    {
        for (size_t i = 0; i < num_queries; ++i)
        {
            generate_random_floats_avx512(query_vectors[i], 250, rng); // Traite 250 floats par ligne
        }
    }

    // Génération d'une instance de VSOChallenge
    Workspace_vs *generate_instance_vs(const uint64_t *seeds, const VSODifficulty *difficulty)
    {
        RngArrayNative *rng = rng_array_native_new(seeds);

        std::thread::id thread_id = std::this_thread::get_id();
        std::call_once(init_flag, initWorkspace_vs);

        int workspace_id = -1;
        Workspace_vs *workspace_ptr = nullptr;
        while (workspace_ptr == nullptr)
        {
            int expected = 0;
            workspace_id = (next_Workspace_vs_index++) % nb_Workspace_vs;

            if ((*workspaces_vs)[workspace_id]->in_use.compare_exchange_strong(expected, 1))
            {
                workspace_ptr = (*workspaces_vs)[workspace_id];
                break;
            }
        }

        Workspace_vs &workspace = *workspace_ptr;
        workspace.solution->len = 0;
        // std::cout << "ThreadId = " << thread_id << " ==> Choose Workspace Id = " << workspace_id << std::endl;

        // Génération de la base de données vectorielle
        // float **vector_database = workspace.vector_database;
        // for (size_t i = 0; i < 100000; ++i)
        // {
        //     for (size_t j = 0; j < 250; ++j)
        //     {
        //         vector_database[i][j] = rng_array_native_sample_uniform32(rng, 0.0, 1.0);
        //     }
        // }

        // // Génération des vecteurs de requête
        // float **query_vectors = workspace.query_vectors;
        // for (size_t i = 0; i < difficulty->num_queries; ++i)
        // {
        //     for (size_t j = 0; j < 250; ++j)
        //     {
        //         query_vectors[i][j] = rng_array_native_sample_uniform32(rng, 0.0, 1.0);
        //     }
        // }
        generate_vector_database_avx512(workspace.vector_database, 100000, rng);
        generate_query_vectors_avx512(workspace.query_vectors, difficulty->num_queries, rng);

        // Calcul de la distance maximale
        float max_distance = 6.0f - static_cast<float>(difficulty->better_than_baseline) / 1000.0f;

        // Création du challenge
        VSOChallenge *challenge = workspace.challenge;
        std::copy(seeds, seeds + 8, ((uint64_t *)challenge->seeds));
        challenge->difficulty.better_than_baseline = difficulty->better_than_baseline;
        challenge->difficulty.num_queries = difficulty->num_queries;
        challenge->vector_database = workspace.vector_database;
        challenge->vector_database_size = 100000; // Taille du vector_database
        challenge->query_vectors = workspace.query_vectors;
        challenge->query_vectors_size = difficulty->num_queries;
        challenge->max_distance = max_distance;

        return workspace_ptr;
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
}
