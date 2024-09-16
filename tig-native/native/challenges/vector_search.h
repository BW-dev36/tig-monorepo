#ifndef VECTOR_SEARCH_H
#define VECTOR_SEARCH_H

#include <vector>
#include <array>
#include <cstdint>
#include <stdint.h>
#include <stddef.h>
#include <atomic>

extern "C"
{

    // Structure VSODifficulty
    struct VSODifficulty
    {
        uint32_t num_queries;
        uint32_t better_than_baseline;
    };

    // Structure VSOChallenge avec pointeurs bruts
    struct VSOChallenge
    {
        uint64_t seeds[8];
        VSODifficulty difficulty;
        float **vector_database; // Pointeurs vers des tableaux de floats
        float **query_vectors;
        size_t vector_database_size;
        size_t query_vectors_size;
        float max_distance;
    };

    // Structure VSOSolution
    struct VSOSolution
    {
        size_t *indexes;
        size_t len;
    };

    class Workspace_vs
    {
    public:
        
        float **vector_database;
        float **query_vectors;
        VSOChallenge *challenge;
        VSOSolution *solution;

        std::atomic<int> in_use;

        Workspace_vs();

        void InitDeviceAllocation();


        ~Workspace_vs();
    };

    // Fonctions externes
    Workspace_vs *generate_instance_vs(const uint64_t * seeds, const VSODifficulty *difficulty);
    unsigned int verify_solution_vs(const VSOChallenge *challenge, const VSOSolution *solution);
    void free_vso_challenge(VSOChallenge *challenge);

    void solve_optimax_cpp(const VSOChallenge *challenge, VSOSolution *solution);
    unsigned int solve_optimax_cpp_full(const uint64_t * seeds, const VSODifficulty *difficulty);
    
}

#endif // VECTOR_SEARCH_H
