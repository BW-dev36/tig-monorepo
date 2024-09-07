#include <iostream>
#include <cstdlib>

extern "C" {
    typedef struct StdRng StdRng;

    StdRng* create_rng(uint64_t seed);
    bool gen_bool(StdRng* rng, double probability);
    int gen_range(StdRng* rng, int min, int max);
    void destroy_rng(StdRng* rng);
}

int main() {
    uint64_t seed = 12345;
    StdRng* rng = create_rng(seed);

    if (rng == nullptr) {
        std::cerr << "Failed to create RNG" << std::endl;
        return 1;
    }

    bool random_bool = gen_bool(rng, 0.5);
    std::cout << "Random boolean: " << random_bool << std::endl;

    int random_range = gen_range(rng, 0, 100);
    std::cout << "Random range: " << random_range << std::endl;

    destroy_rng(rng);
    return 0;
}
