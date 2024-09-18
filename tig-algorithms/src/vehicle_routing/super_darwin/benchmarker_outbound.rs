/*!
Copyright 2024 ByteBandit

Licensed under the TIG Inbound Game License v1.0 or (at your option) any later
version (the "License"); you may not use this file except in compliance with the
License. You may obtain a copy of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use std::collections::HashSet;
use tig_challenges::vehicle_routing::*;

const POPULATION_SIZE: usize = 50;
const ROUNDS: usize = 200;
const INTERNAL_ROUNDS: usize = 5;
const MUTATION_PROBABILITY: f64 = 0.2;
const PERTURBATION: i32 = 20;

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let max_dist: f32 = challenge.distance_matrix[0].iter().sum::<i32>() as f32;
    let p = challenge.max_total_distance as f32 / max_dist;
    if p < 0.5 {
        return Ok(None);
    }

    let mut rng = StdRng::seed_from_u64(challenge.seeds[0] as u64);
    let mut population = generate_initial_population(challenge, &mut rng);

    let mut nb_iter = 0;
    let mut best_routes = population[0].0.clone();
    let mut best_cost = population[0].1;

    while nb_iter < ROUNDS {
        let parent1 = select(&population, &mut rng);
        let parent2 = select(&population, &mut rng);

        for _ in 0..INTERNAL_ROUNDS {
            let (mut child1, mut child2) =
                crossover_and_mutate(&parent1, &parent2, challenge, &mut rng);

            child1 = optimize_with_2_opt(child1, &challenge.distance_matrix);
            child2 = optimize_with_2_opt(child2, &challenge.distance_matrix);

            let child1_cost = compute_total_cost(&child1, &challenge.distance_matrix);
            let child2_cost = compute_total_cost(&child2, &challenge.distance_matrix);

            population.push((child1, child1_cost));
            population.push((child2, child2_cost));
        }

        population.sort_by_key(|&(_, cost)| cost);
        //    population.truncate(POPULATION_SIZE);

        if population[0].1 < best_cost && is_valid_routes(challenge, &population[0].0) {
            best_routes = population[0].0.clone();
            best_cost = population[0].1;
            nb_iter = 1;
        } else {
            nb_iter += 1;
            if nb_iter % (ROUNDS / 5) == 0 {
                population = generate_new_population(population, challenge, &mut rng);
            }
        }
    }

    Ok(Some(Solution {
        routes: best_routes,
    }))
}

fn generate_initial_population(
    challenge: &Challenge,
    rng: &mut StdRng,
) -> Vec<(Vec<Vec<usize>>, i32)> {
    let mut population: Vec<(Vec<Vec<usize>>, i32)> = Vec::with_capacity(POPULATION_SIZE);
    for run in 0..POPULATION_SIZE {
        let random_routes = if run == 0 {
            generate_routes(challenge, true, rng)
        } else {
            generate_routes(challenge, false, rng)
        };
        let random_routes_cost = compute_total_cost(&random_routes, &challenge.distance_matrix);
        population.push((random_routes, random_routes_cost));
    }
    population.sort_by_key(|&(_, cost)| cost);
    population
}

fn generate_new_population(
    mut population: Vec<(Vec<Vec<usize>>, i32)>,
    challenge: &Challenge,
    rng: &mut StdRng,
) -> Vec<(Vec<Vec<usize>>, i32)> {
    let mut unique_population = Vec::with_capacity(population.len());
    let mut seen_routes = HashSet::new();

    population.shuffle(rng);
    population.truncate(POPULATION_SIZE / 2);

    for (routes, cost) in population {
        let flat: Vec<usize> = routes
            .iter()
            .flatten()
            .filter_map(|&x| if x != 0 { Some(x) } else { None })
            .collect();
        if !seen_routes.contains(&flat) {
            seen_routes.insert(flat);
            unique_population.push((routes, cost));
        }
    }

    while unique_population.len() < POPULATION_SIZE {
        let new_routes = generate_routes(challenge, true, rng);
        let new_routes_cost = compute_total_cost(&new_routes, &challenge.distance_matrix);
        unique_population.push((new_routes, new_routes_cost));
    }

    unique_population.sort_by_key(|&(_, cost)| cost);
    unique_population
}

fn select(population: &Vec<(Vec<Vec<usize>>, i32)>, rng: &mut StdRng) -> Vec<Vec<usize>> {
    let candidate1 = &population[rng.gen_range(0..population.len())];
    //let candidate2 = &population[rng.gen_range(0..population.len())];

    /*if candidate1.1 < candidate2.1 {
        candidate1.0.clone()
    } else {
        candidate2.0.clone()
    }*/

    candidate1.0.clone()
}

fn crossover_and_mutate(
    parent1: &Vec<Vec<usize>>,
    parent2: &Vec<Vec<usize>>,
    challenge: &Challenge,
    rng: &mut StdRng,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let mut child1 = crossover_gtbcx(parent1, parent2, challenge, rng);
    let mut child2 = crossover_gtbcx(parent2, parent1, challenge, rng);

    mutate(&mut child1, rng);
    mutate(&mut child2, rng);

    (child1, child2)
}

fn crossover_gtbcx(
    parent1: &Vec<Vec<usize>>,
    parent2: &Vec<Vec<usize>>,
    challenge: &Challenge,
    rng: &mut StdRng,
) -> Vec<Vec<usize>> {
    let parent1_flat: Vec<usize> = parent1
        .iter()
        .flatten()
        .filter(|&&x| x != 0)
        .cloned()
        .collect();

    let parent2_flat: Vec<usize> = parent2
        .iter()
        .flatten()
        .filter(|&&x| x != 0)
        .cloned()
        .collect();

    let idx1 = rng.gen_range(0..parent1_flat.len() - 1);
    let idx2 = rng.gen_range(0..parent2_flat.len() - 1);

    let selected_customers_from_p1 = vec![parent1_flat[idx1], parent1_flat[idx1 + 1]];
    let selected_customers_from_p2 = vec![parent2_flat[idx2], parent2_flat[idx2 + 1]];

    let mut partial_tour1: Vec<usize> = parent1_flat
        .into_iter()
        .filter(|&c| !selected_customers_from_p2.contains(&c))
        .collect();

    let mut partial_tour2: Vec<usize> = parent2_flat
        .into_iter()
        .filter(|&c| !selected_customers_from_p1.contains(&c))
        .collect();

    insert_customers_stochastically(&mut partial_tour1, &selected_customers_from_p2, rng);
    insert_customers_stochastically(&mut partial_tour2, &selected_customers_from_p1, rng);

    let child1_routes = reconstruct_routes(&mut partial_tour1, challenge);
    let child2_routes = reconstruct_routes(&mut partial_tour2, challenge);

    if compute_total_cost(&child1_routes, &challenge.distance_matrix)
        < compute_total_cost(&child2_routes, &challenge.distance_matrix)
    {
        child1_routes
    } else {
        child2_routes
    }
}

fn insert_customers_stochastically(
    partial_tour: &mut Vec<usize>,
    customers_to_insert: &[usize],
    rng: &mut StdRng,
) {
    for &customer in customers_to_insert {
        let position = rng.gen_range(1..=partial_tour.len());
        partial_tour.insert(position, customer);
    }
}

fn reconstruct_routes(giant_tour: &mut Vec<usize>, challenge: &Challenge) -> Vec<Vec<usize>> {
    let mut routes: Vec<Vec<usize>> = Vec::new();
    let mut current_route = vec![0];
    let mut current_capacity = challenge.max_capacity;

    for &customer in giant_tour.iter() {
        let demand = challenge.demands[customer];
        if demand <= current_capacity {
            current_route.push(customer);
            current_capacity -= demand;
        } else {
            current_route.push(0);
            routes.push(current_route);
            current_route = vec![0, customer];
            current_capacity = challenge.max_capacity - demand;
        }
    }

    current_route.push(0);
    routes.push(current_route);
    routes
}

fn mutate(routes: &mut Vec<Vec<usize>>, rng: &mut StdRng) {
    for route in routes {
        if rng.gen_bool(MUTATION_PROBABILITY) {
            let mutation_type = rng.gen_range(0..4);
            match mutation_type {
                0 => {
                    let idx1 = rng.gen_range(1..route.len() - 1);
                    let idx2 = rng.gen_range(1..route.len() - 1);
                    route.swap(idx1, idx2);
                }
                1 => {
                    let start = rng.gen_range(1..route.len() - 1);
                    let end = rng.gen_range(start..route.len() - 1);
                    let mut sublist: Vec<_> = route[start..=end].to_vec();
                    sublist.shuffle(rng);
                    for (i, &val) in sublist.iter().enumerate() {
                        route[start + i] = val;
                    }
                }
                2 => {
                    let start = rng.gen_range(1..route.len() - 1);
                    let end = rng.gen_range(start..route.len() - 1);
                    route[start..=end].reverse();
                }
                3 => {
                    let k = rng.gen_range(1..route.len() - 1);
                    let l = rng.gen_range(1..route.len() - 1);
                    let customer = route.remove(l);
                    route.insert(k, customer);
                }
                _ => {}
            }
        }
    }
}

fn is_valid_routes(challenge: &Challenge, routes: &Vec<Vec<usize>>) -> bool {
    for route in routes {
        let mut capacity = challenge.max_capacity;
        for &node in &route[1..route.len() - 1] {
            let current_capacity = challenge.demands[node];
            if current_capacity > capacity {
                return false;
            }
            capacity -= current_capacity;
        }
    }
    true
}

/************************************* */

fn generate_routes(
    challenge: &Challenge,
    apply_perturbation: bool,
    rng: &mut StdRng,
) -> Vec<Vec<usize>> {
    let d = &challenge.distance_matrix;
    let c = challenge.max_capacity;
    let n = challenge.difficulty.num_nodes;

    let mut scores: Vec<(i32, usize, usize)> = Vec::with_capacity((n * (n - 1)) / 2);
    for i in 1..n {
        let d_i0 = d[i][0];
        for j in (i + 1)..n {
            let score = d_i0 + d[0][j] - d[i][j];
            let perturbed_score = if apply_perturbation {
                score + rng.gen_range(-PERTURBATION..PERTURBATION)
            } else {
                score
            };
            scores.push((perturbed_score, i, j));
        }
    }
    scores.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    let mut routes: Vec<Option<Vec<usize>>> = (0..n).map(|i| Some(vec![i])).collect();
    routes[0] = None;
    let mut route_demands: Vec<i32> = challenge.demands.clone();

    for (s, i, j) in scores {
        if s < 0 {
            break;
        }

        if routes[i].is_none() || routes[j].is_none() {
            continue;
        }

        let (left_route, right_route) = (routes[i].as_ref().unwrap(), routes[j].as_ref().unwrap());

        let (left_startnode, left_endnode) = (left_route[0], *left_route.last().unwrap());
        let (right_startnode, right_endnode) = (right_route[0], *right_route.last().unwrap());
        let merged_demand = route_demands[left_startnode] + route_demands[right_startnode];

        if left_startnode == right_startnode || merged_demand > c {
            continue;
        }

        let mut left_route = routes[i].take().unwrap();
        let mut right_route = routes[j].take().unwrap();
        routes[left_startnode] = None;
        routes[right_startnode] = None;
        routes[left_endnode] = None;
        routes[right_endnode] = None;

        if left_startnode == i {
            left_route.reverse();
        }
        if right_endnode == j {
            right_route.reverse();
        }

        let mut new_route = left_route;
        new_route.extend(right_route);

        let (start, end) = (*new_route.first().unwrap(), *new_route.last().unwrap());
        routes[start] = Some(new_route.clone());
        routes[end] = Some(new_route);
        route_demands[start] = merged_demand;
        route_demands[end] = merged_demand;
    }

    let mut final_routes = Vec::with_capacity(routes.len());
    for (i, opt_route) in routes.into_iter().enumerate() {
        if let Some(mut route) = opt_route {
            if route[0] == i {
                let mut full_route = Vec::with_capacity(route.len() + 2);
                full_route.push(0);
                full_route.append(&mut route);
                full_route.push(0);
                final_routes.push(full_route);
            }
        }
    }

    optimize_with_2_opt(final_routes, &challenge.distance_matrix)
}

/************************************* */

fn optimize_with_2_opt(
    mut routes: Vec<Vec<usize>>,
    distance_matrix: &Vec<Vec<i32>>,
) -> Vec<Vec<usize>> {
    let mut improved = true;

    while improved {
        improved = false;
        for route in &mut routes {
            let route_length = route.len();
            for i in 1..route_length - 2 {
                for j in i + 1..route_length - 1 {
                    let delta = distance_matrix[route[i - 1]][route[j]]
                        + distance_matrix[route[i]][route[j + 1]]
                        - (distance_matrix[route[i - 1]][route[i]]
                            + distance_matrix[route[j]][route[j + 1]]);
                    if delta < 0 {
                        route[i..=j].reverse();
                        improved = true;
                    }
                }
            }
        }
    }

    routes
}

fn compute_total_cost(routes: &Vec<Vec<usize>>, distance_matrix: &Vec<Vec<i32>>) -> i32 {
    routes
        .iter()
        .map(|route| compute_route_cost(route, distance_matrix))
        .sum()
}

fn compute_route_cost(route: &Vec<usize>, distance_matrix: &Vec<Vec<i32>>) -> i32 {
    route
        .windows(2)
        .map(|pair| distance_matrix[pair[0]][pair[1]])
        .sum()
}

/************************************* */

#[cfg(feature = "cuda")]
mod gpu_optimization {
    use super::*;
    use cudarc::driver::*;
    use std::{collections::HashMap, sync::Arc};
    use tig_challenges::CudaKernel;

    pub const KERNEL: Option<CudaKernel> = None;

    pub fn cuda_solve_challenge(
        challenge: &Challenge,
        device: &Arc<CudaDevice>,
        functions: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<Option<Solution>> {
        solve_challenge(challenge)
    }
}

#[cfg(feature = "cuda")]
pub use gpu_optimization::{cuda_solve_challenge, KERNEL};
