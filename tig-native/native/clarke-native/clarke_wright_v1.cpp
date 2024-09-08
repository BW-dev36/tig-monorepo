#include <vector>
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <iostream>

extern "C" {
    struct CWChallenge {
        uint64_t seed;
        int* demands;
        int* distance_matrix;
        int max_total_distance;
        int max_capacity;
        unsigned int num_nodes;
    };

    struct CWSolution {
        int** routes;
        int* route_lengths;
        int num_routes;
    };

    struct Score
    {
        int score;
        size_t i;
        size_t j;
    };

    void solve_clarke_wright_v1_cpp(const CWChallenge* challenge, CWSolution* solution) {
        const int* d = challenge->distance_matrix;
        const int c = challenge->max_capacity;
        const unsigned int n = challenge->num_nodes;
        //std::cout << "Num nodes : " << n << std::endl;
        std::vector<Score> scores;
        for (size_t i = 1; i < n; ++i) {
            for (size_t j = i + 1; j < n; ++j) {
                int score = d[i * n + 0] + d[0 * n + j] - d[i * n + j];
                scores.emplace_back(Score{score, i, j});
            }
        }
        //std::cout << "Is it called ? " << std::endl;
        std::sort(scores.begin(), scores.end(), [](const Score& a, const Score& b) { return a.score > b.score; });

        std::vector<std::vector<size_t>> routes(n);
        for (size_t i = 0; i < n; ++i) {
            routes[i].push_back(i);
        }
        routes[0].clear();

        std::vector<int> route_demands(challenge->demands, challenge->demands + n);

        for (const auto& [s, i, j] : scores) {
            if (s < 0) {
                break;
            }

            if (routes[i].empty() || routes[j].empty()) {
                continue;
            }

            auto left_route = routes[i];
            auto right_route = routes[j];
            size_t left_startnode = left_route.front();
            size_t right_startnode = right_route.front();
            size_t left_endnode = left_route.back();
            size_t right_endnode = right_route.back();
            int merged_demand = route_demands[left_startnode] + route_demands[right_startnode];

            if (left_startnode == right_startnode || merged_demand > c) {
                continue;
            }

            routes[i].clear();
            routes[j].clear();
            routes[left_startnode].clear();
            routes[right_startnode].clear();
            routes[left_endnode].clear();
            routes[right_endnode].clear();

            if (left_startnode == i) {
                std::reverse(left_route.begin(), left_route.end());
                left_startnode = left_route.front();
            }
            if (right_endnode == j) {
                std::reverse(right_route.begin(), right_route.end());
                right_endnode = right_route.back();
            }

            std::vector<size_t> new_route = left_route;
            new_route.insert(new_route.end(), right_route.begin(), right_route.end());

            routes[left_startnode] = new_route;
            routes[right_endnode] = new_route;
            route_demands[left_startnode] = merged_demand;
            route_demands[right_endnode] = merged_demand;
        }

        std::vector<std::vector<size_t>> final_routes;
        for (size_t i = 0; i < routes.size(); ++i) {
            if (!routes[i].empty() && routes[i].front() == i) {
                std::vector<size_t> route = {0};
                route.insert(route.end(), routes[i].begin(), routes[i].end());
                route.push_back(0);
                final_routes.push_back(route);
            }
        }

        solution->num_routes = final_routes.size();
        solution->routes = new int*[solution->num_routes];
        solution->route_lengths = new int[solution->num_routes];

        //std::cout << "Num routes : " << routes.size() << std::endl;
        for (int i = 0; i < solution->num_routes; ++i) {
            solution->route_lengths[i] = final_routes[i].size();
            solution->routes[i] = new int[solution->route_lengths[i]];
            std::copy(final_routes[i].begin(), final_routes[i].end(), solution->routes[i]);
        }
    }
}