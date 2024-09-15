#pragma once 
#include <cstdint>


extern "C" {
    typedef struct StdRng StdRng;

    StdRng* create_rng(uint64_t seed);
    bool gen_bool(StdRng* rng, double probability);
    unsigned int gen_range(StdRng* rng, unsigned int min, unsigned int max);
    void destroy_rng(StdRng* rng);
}
