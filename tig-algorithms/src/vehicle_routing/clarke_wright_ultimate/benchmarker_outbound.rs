use tig_challenges::vehicle_routing::*;

const NUM_PERTURBATIONS: i32 = 30;
const PERTURBATION_LIMIT: i32 = 15;
const POPULATION_COUNT: usize = 20;
const ROUNDS: usize = 30;

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let distance_matrix = &challenge.distance_matrix;
    let num_nodes = challenge.difficulty.num_nodes;
    let mut rng = ChaCha8Rng::seed_from_u64(challenge.seeds[0] as u64);

    let mut best_solution: Option<Solution> = None;
    let mut minimum_cost: i32 = i32::MAX;

    let original_scores = compute_scores(distance_matrix, num_nodes);
    let mut perturbation_limit = PERTURBATION_LIMIT;

    for run in -1..NUM_PERTURBATIONS {
        if run % 2 == 0 {
            perturbation_limit += 2;
        }

        let mut scores = original_scores.clone();
        if run >= 0 {
            for score in &mut scores {
                let perturbation: i32 = rng.gen_range(-perturbation_limit..=perturbation_limit);
                score.0 += perturbation;
            }
            scores.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        }
        let mut solution = generate_initial_solution(challenge, &mut scores);
        solution = optimize_with_2_opt(solution, distance_matrix);
        let total_cost = compute_total_cost(&solution, distance_matrix);

        if total_cost < challenge.max_total_distance {
            return Ok(Some(solution));
        }
        update_best_solution(
            &mut best_solution,
            &mut minimum_cost,
            (solution, total_cost),
        );
    }

    let genetic_solution = genetic_algorithm(&best_solution, minimum_cost, challenge, &mut rng);
    if let Some(best_genetic_solution) = genetic_solution {
        update_best_solution(&mut best_solution, &mut minimum_cost, best_genetic_solution);
    }

    Ok(best_solution)
}

fn compute_scores(distance_matrix: &Vec<Vec<i32>>, num_nodes: usize) -> Vec<(i32, usize, usize)> {
    let mut scores = Vec::with_capacity((num_nodes * (num_nodes - 1)) / 2);
    for i in 1..num_nodes {
        let distance_to_depot = distance_matrix[i][0];
        for j in (i + 1)..num_nodes {
            let score = distance_to_depot + distance_matrix[0][j] - distance_matrix[i][j];
            scores.push((score, i, j));
        }
    }
    scores.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    scores
}

fn generate_initial_solution(
    challenge: &Challenge,
    scores: &mut Vec<(i32, usize, usize)>,
) -> Solution {
    let max_capacity = challenge.max_capacity;
    let num_nodes = challenge.difficulty.num_nodes;

    let mut routes: Vec<Option<Vec<usize>>> = (0..num_nodes).map(|i| Some(vec![i])).collect();
    routes[0] = None;
    let mut route_demands: Vec<i32> = challenge.demands.clone();

    for &mut (score, i, j) in scores {
        if score < 0 {
            break;
        }

        if routes[i].is_none() || routes[j].is_none() {
            continue;
        }

        let (route_i, route_j) = (routes[i].as_ref().unwrap(), routes[j].as_ref().unwrap());

        let (start_i, end_i) = (route_i[0], *route_i.last().unwrap());
        let (start_j, end_j) = (route_j[0], *route_j.last().unwrap());
        let combined_demand = route_demands[start_i] + route_demands[start_j];

        if start_i == start_j || combined_demand > max_capacity {
            continue;
        }

        let mut route_i = routes[i].take().unwrap();
        let mut route_j = routes[j].take().unwrap();
        routes[start_i] = None;
        routes[start_j] = None;
        routes[end_i] = None;
        routes[end_j] = None;

        if start_i == i {
            route_i.reverse();
        }
        if end_j == j {
            route_j.reverse();
        }

        let mut merged_route = route_i;
        merged_route.extend(route_j);

        let (new_start, new_end) = (
            *merged_route.first().unwrap(),
            *merged_route.last().unwrap(),
        );
        routes[new_start] = Some(merged_route.clone());
        routes[new_end] = Some(merged_route);
        route_demands[new_start] = combined_demand;
        route_demands[new_end] = combined_demand;
    }

    let final_routes: Vec<Vec<usize>> = routes
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt_route)| {
            if let Some(mut route) = opt_route {
                if route[0] == i {
                    let mut full_route = Vec::with_capacity(route.len() + 2);
                    full_route.push(0);
                    full_route.append(&mut route);
                    full_route.push(0);
                    Some(full_route)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    Solution {
        routes: final_routes,
    }
}

fn genetic_algorithm(
    best_solution: &Option<Solution>,
    best_cost: i32,
    challenge: &Challenge,
    rng: &mut ChaCha8Rng,
) -> Option<(Solution, i32)> {
    if let Some(first_solution) = best_solution {
        let mut population: Vec<(Solution, i32)> = Vec::with_capacity(POPULATION_COUNT);

        population.push((
            Solution {
                routes: first_solution.routes.clone(),
            },
            best_cost,
        ));

        for _ in 0..POPULATION_COUNT {
            let random_solution = generate_random_solution(challenge, rng);
            let random_solution_cost =
                compute_total_cost(&random_solution, &challenge.distance_matrix);
            population.push((random_solution, random_solution_cost));
        }
        population.sort_by_key(|&(_, cost)| cost);

        for _ in 0..ROUNDS {
            let parent1 = &population[0].0;
            let parent2 = &population[1].0;

            let mut offspring: Vec<(Solution, i32)> = Vec::new();
            for _ in 0..((POPULATION_COUNT - 2) / 2) {
                let (child1, child2) = crossover_and_mutate(parent1, parent2, challenge, rng);
                let child1_cost = compute_total_cost(&child1, &challenge.distance_matrix);
                let child2_cost = compute_total_cost(&child2, &challenge.distance_matrix);
                offspring.push((child1, child1_cost));
                offspring.push((child2, child2_cost));
            }

            population.extend(offspring);
            population.sort_by_key(|&(_, cost)| cost);
            population.truncate(2);
        }

        return population.into_iter().min_by_key(|&(_, cost)| cost);
    }
    None
}

fn generate_random_solution(challenge: &Challenge, rng: &mut ChaCha8Rng) -> Solution {
    let num_nodes = challenge.difficulty.num_nodes;
    let mut routes: Vec<Vec<usize>> = Vec::with_capacity(num_nodes / 2);
    let mut unvisited: Vec<usize> = (1..num_nodes).collect();

    fn shuffle<T>(vec: &mut Vec<T>, rng: &mut ChaCha8Rng) {
        let len = vec.len();
        for i in (1..len).rev() {
            let j = rng.gen_range(0..=i);
            if i != j {
                vec.swap(i, j);
            }
        }
    }
    shuffle(&mut unvisited, rng);

    let mut current_route: Vec<usize> = Vec::with_capacity(num_nodes);
    current_route.push(0);

    let mut current_capacity = challenge.max_capacity;

    for &node in &unvisited {
        let demand = challenge.demands[node];
        if demand <= current_capacity {
            current_route.push(node);
            current_capacity -= demand;
        } else {
            current_route.push(0);
            routes.push(current_route);

            current_route = Vec::with_capacity(num_nodes);
            current_route.push(0);
            current_route.push(node);
            current_capacity = challenge.max_capacity - demand;
        }
    }

    current_route.push(0);
    routes.push(current_route);

    Solution { routes }
}

fn crossover_and_mutate(
    parent1: &Solution,
    parent2: &Solution,
    challenge: &Challenge,
    rng: &mut ChaCha8Rng,
) -> (Solution, Solution) {
    let split_index = rng.gen_range(1..=parent1.routes.len() - 2);

    let mut child1_routes = parent1.routes[..split_index].to_vec();
    child1_routes.extend_from_slice(&parent2.routes[split_index..]);

    let mut child2_routes = parent2.routes[..split_index].to_vec();
    child2_routes.extend_from_slice(&parent1.routes[split_index..]);

    let mut child1 = Solution {
        routes: child1_routes,
    };
    let mut child2 = Solution {
        routes: child2_routes,
    };

    child1 = mutate(child1, &challenge.distance_matrix, rng);
    child2 = mutate(child2, &challenge.distance_matrix, rng);

    (child1, child2)
}

fn mutate(solution: Solution, distance_matrix: &Vec<Vec<i32>>, rng: &mut ChaCha8Rng) -> Solution {
    let mut mutated_solution = Solution {
        routes: (solution.routes.clone()),
    };

    let mut mutated = false;
    for route in &mut mutated_solution.routes {
        if route.len() > 2 {
            for i in 1..route.len() - 1 {
                if rng.gen_range(1..=5) == 1 {
                    let swap_idx = rng.gen_range(1..=route.len() - 2);
                    route.swap(i, swap_idx);
                    mutated = true;
                }
            }
        }
    }

    if mutated {
        return optimize_with_2_opt(mutated_solution, distance_matrix);
    }
    mutated_solution
}

fn optimize_with_2_opt(mut solution: Solution, distance_matrix: &Vec<Vec<i32>>) -> Solution {
    let mut improved = true;

    while improved {
        improved = false;
        for route in &mut solution.routes {
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

    solution
}

fn compute_total_cost(solution: &Solution, distance_matrix: &Vec<Vec<i32>>) -> i32 {
    solution
        .routes
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

fn update_best_solution(
    best_solution: &mut Option<Solution>,
    minimum_cost: &mut i32,
    candidate_solution: (Solution, i32),
) {
    if candidate_solution.1 < *minimum_cost {
        *minimum_cost = candidate_solution.1;
        *best_solution = Some(candidate_solution.0);
    }
}

/************************************ */
/************************************ */
/************************************ */

pub struct ChaCha8Rng {
    state: [u32; 16],
    index: usize,
}

impl ChaCha8Rng {
    pub fn seed_from_u64(seed: u64) -> Self {
        let mut state = [0u32; 16];
        state[0] = 0x61707865;
        state[1] = 0x3320646e;
        state[2] = 0x79622d32;
        state[3] = 0x6b206574;
        state[4] = (seed & 0xFFFFFFFF) as u32;
        state[5] = (seed >> 32) as u32;
        ChaCha8Rng { state, index: 0 }
    }

    fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(16);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(12);

        state[a] = state[a].wrapping_add(state[b]);
        state[d] ^= state[a];
        state[d] = state[d].rotate_left(8);

        state[c] = state[c].wrapping_add(state[d]);
        state[b] ^= state[c];
        state[b] = state[b].rotate_left(7);
    }

    fn chacha8_rounds(&mut self) {
        for _ in 0..8 {
            Self::quarter_round(&mut self.state, 0, 4, 8, 12);
            Self::quarter_round(&mut self.state, 1, 5, 9, 13);
            Self::quarter_round(&mut self.state, 2, 6, 10, 14);
            Self::quarter_round(&mut self.state, 3, 7, 11, 15);
            Self::quarter_round(&mut self.state, 0, 5, 10, 15);
            Self::quarter_round(&mut self.state, 1, 6, 11, 12);
            Self::quarter_round(&mut self.state, 2, 7, 8, 13);
            Self::quarter_round(&mut self.state, 3, 4, 9, 14);
        }
    }

    pub fn gen_u32(&mut self) -> u32 {
        if self.index == 0 {
            self.chacha8_rounds();
        }
        let result = self.state[self.index];
        self.index = (self.index + 1) % 16;
        result
    }

    pub fn gen_range<T>(&mut self, range: std::ops::RangeInclusive<T>) -> T
    where
        T: Copy + PartialOrd + From<u8> + TryFrom<i64> + TryInto<i64>,
    {
        let low: i64 = (*range.start()).try_into().ok().unwrap();
        let high: i64 = (*range.end()).try_into().ok().unwrap();
        let diff = high - low;
        let random_value = (self.gen_u32() as i64) % (diff + 1);
        let result: i64 = low + random_value;
        result.try_into().ok().unwrap()
    }
}

/************************************ */
/************************************ */
/************************************ */

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
