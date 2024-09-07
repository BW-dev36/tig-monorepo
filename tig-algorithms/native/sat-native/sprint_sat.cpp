#include <vector>
#include <unordered_map>
#include <random>
#include <optional>
#include <algorithm>
#include <cstring>
#include <iostream>
#include "structure.h"

extern "C" {

    std::uniform_real_distribution<> dist(0.0, 1.0);
    size_t get_random_index(std::mt19937 &rng, size_t upper_bound) {
        std::uniform_int_distribution<> dist(0, upper_bound - 1);
        return dist(rng);
    }

    size_t get_v(std::mt19937 &rng, size_t min_sad, const std::vector<size_t>& v_min_sad, const std::vector<int>& c) {
        if (min_sad == 0) {
            if (v_min_sad.size() == 1) {
                return v_min_sad[0];
            } else {
                return v_min_sad[get_random_index(rng, v_min_sad.size())];
            }
        } else {
            if (dist(rng) < 0.5) {
                int l = c[get_random_index(rng, c.size())];
                return std::abs(l) - 1;
            } else {
                return v_min_sad[get_random_index(rng, v_min_sad.size())];
            }
        }
    }

void update_residuals(
    std::vector<size_t>& residual_,
    std::unordered_map<size_t, size_t>& residual_indices,
    std::vector<size_t>& num_good_so_far,
    const std::vector<size_t>& n_clauses_v,
    const std::vector<size_t>& p_clauses_v,
    std::vector<bool>& variables,
    size_t v
) {
    if (variables[v]) {
        for (auto c : n_clauses_v) {
            num_good_so_far[c] += 1;
            if (num_good_so_far[c] == 1) {
                auto it = residual_indices.find(c);
                if (it != residual_indices.end()) {
                    size_t i = it->second;
                    residual_indices.erase(it);
                    size_t last = residual_.back();
                    residual_.pop_back();
                    if (i < residual_.size()) {
                        residual_[i] = last;
                        residual_indices[last] = i;
                    }
                }
            }
        }
        for (auto c : p_clauses_v) {
            if (num_good_so_far[c] == 1) {
                residual_.push_back(c);
                residual_indices[c] = residual_.size() - 1;
            }
            num_good_so_far[c] -= 1;
        }
    } else {
        for (auto c : n_clauses_v) {
            if (num_good_so_far[c] == 1) {
                residual_.push_back(c);
                residual_indices[c] = residual_.size() - 1;
            }
            num_good_so_far[c] -= 1;
        }
        for (auto c : p_clauses_v) {
            num_good_so_far[c] += 1;
            if (num_good_so_far[c] == 1) {
                auto it = residual_indices.find(c);
                if (it != residual_indices.end()) {
                    size_t i = it->second;
                    residual_indices.erase(it);
                    size_t last = residual_.back();
                    residual_.pop_back();
                    if (i < residual_.size()) {
                        residual_[i] = last;
                        residual_indices[last] = i;
                    }
                }
            }
        }
    }
    variables[v] = !variables[v];
}

    void solve_sprint_sat_v1_cpp(const SATChallenge* challenge, SATSolution* solution) {
        std::mt19937 rng(challenge->seed);
        std::vector<bool> p_single(challenge->num_variables, false);
        std::vector<bool> n_single(challenge->num_variables, false);

        std::vector<std::vector<int>> clauses;
        clauses.reserve(challenge->num_clauses);
        int* clause_ptr = challenge->clauses;
        for (int i = 0; i < challenge->num_clauses; ++i) {
            clauses.emplace_back(clause_ptr, clause_ptr + challenge->clause_lengths[i]);
            clause_ptr += challenge->clause_lengths[i];
        }

        std::vector<std::vector<int>> clauses_;
        bool dead = false;

        while (!dead) {
            bool done = true;
            for (const auto& c : clauses) {
                std::vector<int> c_;
                c_.reserve(c.size());
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
                    clauses_.push_back(c_);
                }
            }
            if (done) {
                break;
            } else {
                clauses = std::move(clauses_);
                clauses_.clear();
                clauses_.reserve(clauses.size());
            }
        }

        if (dead) {
            return;
        }

        int num_variables = challenge->num_variables;
        int num_clauses = clauses.size();

        std::vector<std::vector<size_t>> p_clauses(num_variables);
        std::vector<std::vector<size_t>> n_clauses(num_variables);
        std::vector<bool> variables(num_variables, false);

        for (int v = 0; v < num_variables; ++v) {
            if (p_single[v]) {
                variables[v] = true;
            } else if (n_single[v]) {
                variables[v] = false;
            } else {
                variables[v] = rng() % 2 == 0;
            }
        }

        std::vector<size_t> num_good_so_far(num_clauses, 0);
        for (size_t i = 0; i < clauses.size(); ++i) {
            for (int l : clauses[i]) {
                size_t var = std::abs(l) - 1;
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

        std::vector<size_t> residual_;
        std::unordered_map<size_t, size_t> residual_indices;

        for (size_t i = 0; i < num_good_so_far.size(); ++i) {
            if (num_good_so_far[i] == 0) {
                residual_.push_back(i);
                residual_indices[i] = residual_.size() - 1;
            }
        }

        size_t attempts = 0;
        while (true) {
            if (attempts >= num_variables * 25) {
                // Convert std::vector<bool> to std::vector<char>
                for (int i = 0; i < num_variables; ++i) {
                    solution->variables[i] = false;
                }
                return;
            }
            if (!residual_.empty()) {
                size_t i = residual_.front();
                size_t min_sad = clauses.size();
                std::vector<size_t> v_min_sad;
                const auto& c = clauses[i];
                for (int l : c) {
                    size_t sad = 0;
                    size_t var = std::abs(l) - 1;
                    if (variables[var]) {
                        for (size_t ci : p_clauses[var]) {
                            if (num_good_so_far[ci] == 1) {
                                sad++;
                                if (sad > min_sad) {
                                    break;
                                }
                            }
                        }
                    } else {
                        for (size_t ci : n_clauses[var]) {
                            if (num_good_so_far[ci] == 1) {
                                sad++;
                                if (sad > min_sad) {
                                    break;
                                }
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
                size_t v = get_v(rng, min_sad, v_min_sad, c);

                // Update Residual
                update_residuals(residual_, residual_indices, 
                                num_good_so_far, n_clauses[v], 
                                p_clauses[v], variables, v);

            } else {
                break;
            }
            attempts++;
        }

       
        //std::cout << "Nb Variables " << num_variables << std::endl;
        // Convert std::vector<bool> to std::vector<char>
        for (int i = 0; i < num_variables; ++i) {
            solution->variables[i] = variables[i];
        }

        //std::memcpy(solution->variables, variables.data(), num_variables * sizeof(bool));
    }
}
