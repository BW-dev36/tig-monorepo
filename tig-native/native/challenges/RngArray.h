#pragma once
#include <random>
#include <array>
#include <cstdint>

class RngArray {
public:
    RngArray(const uint64_t * seeds);

    // Sélectionne aléatoirement un générateur et le retourne
    std::mt19937& get_mut();

private:
    std::array<std::mt19937, 8> rngs;  // Tableau de 8 générateurs Mersenne Twister
    int index;  // Index du générateur courant
};

// Declaration of the Rust functions
extern "C" {
    typedef struct RngArrayNative RngArrayNative;

    RngArrayNative* rng_array_native_new(const uint64_t* seeds);
    void rng_array_native_free(RngArrayNative* ptr);
    double rng_array_native_sample_uniform64(RngArrayNative* ptr, double low, double high);
    float rng_array_native_sample_uniform32(RngArrayNative* ptr, float low, float high);
}
