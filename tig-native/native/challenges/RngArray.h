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
