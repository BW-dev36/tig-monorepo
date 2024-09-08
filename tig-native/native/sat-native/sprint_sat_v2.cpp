#include <vector>
#include <unordered_map>
#include <random>
#include <algorithm>
#include <cstring>
#include "structure.h"



extern "C" {
    typedef struct StdRng StdRng;

    StdRng* create_rng(uint64_t seed);
    bool gen_bool(StdRng* rng, double probability);
    unsigned int gen_range(StdRng* rng, unsigned int min, unsigned int max);
    void destroy_rng(StdRng* rng);
}


extern "C" void solve_sprint_sat_v2_cpp(const SATChallenge* challenge, SATSolution* solution) {
    StdRng* rng = create_rng(challenge->seed);

    std::vector<bool> p_single(challenge->num_variables, false);
    std::vector<bool> n_single(challenge->num_variables, false);

    std::vector<std::vector<int>> clauses;

    
    for (int i = 0, offset = 0; i < challenge->num_clauses; ++i) {
        clauses.emplace_back(challenge->clauses + offset, challenge->clauses + offset + challenge->clause_lengths[i]);
        offset += challenge->clause_lengths[i];
    }

    bool dead = false;

    while (!dead) {
        bool done = true;
        std::vector<std::vector<int>> new_clauses;
        for (const auto& c : clauses) {
            std::vector<int> c_;
            bool skip = false;

            for (size_t i = 0; i < c.size(); ++i) {
                int l = c[i];
                if ((p_single[std::abs(l) - 1] && l > 0) ||
                    (n_single[std::abs(l) - 1] && l < 0) ||
                    std::find(c.begin() + i + 1, c.end(), -l) != c.end()) {
                    skip = true;
                    break;
                } else if (p_single[std::abs(l) - 1] ||
                           n_single[std::abs(l) - 1] ||
                           std::find(c.begin() + i + 1, c.end(), l) != c.end()) {
                    done = false;
                    continue;
                } else {
                    c_.push_back(l);
                }
            }

            if (skip) {
                done = false;
                continue;
            }

            if (c_.size() == 1) {
                done = false;
                int l = c_[0];
                if (l > 0) {
                    if (n_single[std::abs(l) - 1]) {
                        dead = true;
                        break;
                    } else {
                        p_single[std::abs(l) - 1] = true;
                    }
                } else {
                    if (p_single[std::abs(l) - 1]) {
                        dead = true;
                        break;
                    } else {
                        n_single[std::abs(l) - 1] = true;
                    }
                }
            } else if (c_.empty()) {
                dead = true;
                break;
            } else {
                new_clauses.push_back(c_);
            }
        }

        if (done) break;
        clauses = std::move(new_clauses);
    }

    int num_variables = challenge->num_variables;

    if (dead) {
        std::memset(solution->variables, false, num_variables * sizeof(bool));
        solution->num_variables = 0;
        return;
    }

    
    int num_clauses = clauses.size();

    std::vector<std::vector<int>> p_clauses(num_variables);
    std::vector<std::vector<int>> n_clauses(num_variables);

    std::vector<bool> variables(num_variables);
    for (int v = 0; v < num_variables; ++v) {
        if (p_single[v]) {
            variables[v] = true;
        } else if (n_single[v]) {
            variables[v] = false;
        } else {
            variables[v] = gen_bool(rng, 0.5);
        }
    }

    std::vector<int> num_good_so_far(num_clauses, 0);

    
    for (int i = 0; i < num_clauses; ++i) {
        for (int l : clauses[i]) {
            int var = std::abs(l) - 1;
            if (l > 0) {
                if (p_clauses[var].capacity() == 0) {
                    p_clauses[var].reserve(clauses.size() / num_variables + 1);
                }
            } else {
                if (n_clauses[var].capacity() == 0) {
                    n_clauses[var].reserve(clauses.size() / num_variables + 1);
                }
            }
        }
    }

    for (int i = 0; i < num_clauses; ++i) {
        for (int l : clauses[i]) {
            int var = std::abs(l) - 1;
            if (l > 0) {
                p_clauses[var].push_back(i);
                if (variables[var]) {
                    num_good_so_far[i]++;
                }
            } else {
                n_clauses[var].push_back(i);
                if (!variables[var]) {
                    num_good_so_far[i]++;
                }
            }
        }
    }

    std::vector<int> residual;
    std::unordered_map<int, int> residual_indices;

    for (int i = 0; i < num_clauses; ++i) {
        if (num_good_so_far[i] == 0) {
            residual_indices[i] = residual.size();
            residual.push_back(i);
        }
    }

    int attempts = 0;
    while (attempts < num_variables * 35 && !residual.empty()) {
        int i = residual[0];
        int min_sad = clauses.size();
        std::vector<int> v_min_sad;

        for (int l : clauses[i]) {
            int sad = 0;
            int var = std::abs(l) - 1;

            if (variables[var]) {
                for (int c : p_clauses[var]) {
                    if (num_good_so_far[c] == 1) {
                        sad++;
                        if (sad > min_sad) break;
                    }
                }
            } else {
                for (int c : n_clauses[var]) {
                    if (num_good_so_far[c] == 1) {
                        sad++;
                        if (sad > min_sad) break;
                    }
                }
            }

            if (sad < min_sad) {
                min_sad = sad;
                v_min_sad = {var};
            } else if (sad == min_sad) {
                v_min_sad.push_back(var);
            }
        }

        int v;
        if (min_sad == 0) {
            if (v_min_sad.size() == 1)
            {
                v = v_min_sad[0];
            }
            else
            {
                //printf("Range from sprint %d %d\n", 0, v_min_sad.size());
                v = v_min_sad[gen_range(rng, 0, v_min_sad.size())];
            }
        } else {
            if (gen_bool(rng, 0.5)) {
                //printf("Range from sprint clause %d %d\n", 0, clauses[i].size());
                int l = clauses[i][gen_range(rng, 0, clauses[i].size())];
                v = std::abs(l) - 1;
            } else {
                //printf("Range from sprint v_min_sad %d %d\n", 0, v_min_sad.size());
                v = v_min_sad[gen_range(rng, 0, v_min_sad.size())];
            }
        }

        if (variables[v]) {
            for (int c : n_clauses[v]) {
                num_good_so_far[c]++;
                if (num_good_so_far[c] == 1) {
                    auto it = residual_indices.find(c);
                    if (it != residual_indices.end()) {
                        int idx = it->second;
                        residual[idx] = residual.back();
                        residual_indices[residual.back()] = idx;
                        residual.pop_back();
                        residual_indices.erase(it);
                    }
                }
            }
            for (int c : p_clauses[v]) {
                if (num_good_so_far[c] == 1) {
                    residual_indices[c] = residual.size();
                    residual.push_back(c);
                }
                num_good_so_far[c]--;
            }
        } else {
            for (int c : n_clauses[v]) {
                if (num_good_so_far[c] == 1) {
                    residual_indices[c] = residual.size();
                    residual.push_back(c);
                }
                num_good_so_far[c]--;
            }
            for (int c : p_clauses[v]) {
                num_good_so_far[c]++;
                if (num_good_so_far[c] == 1) {
                    auto it = residual_indices.find(c);
                    if (it != residual_indices.end()) {
                        int idx = it->second;
                        residual[idx] = residual.back();
                        residual_indices[residual.back()] = idx;
                        residual.pop_back();
                        residual_indices.erase(it);
                    }
                }
            }
        }

        variables[v] = !variables[v];
        attempts++;
    }

    if (residual.empty()) {
        for (int i = 0; i < num_variables; i++)
        {
            solution->variables[i] = variables[i];
            solution->num_variables = num_variables;
        }
    } else {
        std::memset(solution->variables, false, num_variables * sizeof(bool));
        solution->num_variables = 0;
    }
    destroy_rng(rng);
}
