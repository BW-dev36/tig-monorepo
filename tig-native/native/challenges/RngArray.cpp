#include "RngArray.h"

RngArray::RngArray(const uint64_t *seeds)
{
    // Initialisation des générateurs Mersenne Twister avec les seeds fournies
    for (size_t i = 0; i < 8; ++i)
    {
        rngs[i] = std::mt19937(seeds[i]);
    }
    index = 0;
}

// Sélectionne aléatoirement un générateur et le retourne
std::mt19937 &RngArray::get_mut()
{
    // Utiliser le générateur courant pour choisir un nouvel index aléatoire entre 0 et 7
    std::uniform_int_distribution<int> dist(0, 7);
    index = dist(rngs[index]);
    return rngs[index];
}
