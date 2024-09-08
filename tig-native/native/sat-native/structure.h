#pragma once


extern "C" {

    struct SATChallenge {
        uint64_t seed;
        int num_variables;
        int* clauses;
        int* clause_lengths;
        int num_clauses;
    };

    struct SATSolution {
        bool* variables;
        int num_variables;
    };
}