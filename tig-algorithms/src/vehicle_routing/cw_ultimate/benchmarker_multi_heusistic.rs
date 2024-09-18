use anyhow::Result;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::cmp::Ordering;
use std::f64::consts::PI;
use tig_challenges::{vehicle_routing::*, RngArray};

const ROUNDS: usize = 30;

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let distance_matrix = &challenge.distance_matrix;
    let mut rng = StdRng::seed_from_u64(challenge.seeds[0] as u64);
    let mut population: Vec<(Solution, i32)> = Vec::with_capacity(5);

    let cw_solution = generate_clarke_wright_solution(challenge);
    let cw_cost = compute_total_cost(&cw_solution, distance_matrix);
    population.push((cw_solution, cw_cost));

    let greedy_solution = generate_greedy_solution(challenge);
    let greedy_cost = compute_total_cost(&greedy_solution, distance_matrix);
    population.push((greedy_solution, greedy_cost));

    let generate_insertion_heuristic_solution = generate_insertion_heuristic_solution(challenge);
    let generate_insertion_heuristic_cost =
        compute_total_cost(&generate_insertion_heuristic_solution, distance_matrix);
    population.push((
        generate_insertion_heuristic_solution,
        generate_insertion_heuristic_cost,
    ));

    let generate_nearest_neighbor_solution = generate_nearest_neighbor_solution(challenge);
    let generate_nearest_neighbor_cost =
        compute_total_cost(&generate_nearest_neighbor_solution, distance_matrix);
    population.push((
        generate_nearest_neighbor_solution,
        generate_nearest_neighbor_cost,
    ));

    let generate_sweep_solution = generate_sweep_solution(challenge);
    let generate_sweep_cost = compute_total_cost(&generate_sweep_solution, distance_matrix);
    population.push((generate_sweep_solution, generate_sweep_cost));

    let genetic_solution = genetic_algorithm(&mut population, challenge, &mut rng);

    Ok(genetic_solution)
}

fn genetic_algorithm(
    population: &mut Vec<(Solution, i32)>,
    challenge: &Challenge,
    rng: &mut StdRng,
) -> Option<Solution> {
    population.sort_by_key(|&(_, cost)| cost);

    for _ in 0..ROUNDS {
        let parent1 = &population[0].0;
        let parent2 = &population[1].0;

        let mut offspring: Vec<(Solution, i32)> = Vec::new();
        for _ in 0..5 {
            let (child1, child2) = crossover_and_mutate(parent1, parent2, challenge, rng);
            let child1_cost = compute_total_cost(&child1, &challenge.distance_matrix);
            let child2_cost = compute_total_cost(&child2, &challenge.distance_matrix);

            if is_valid_solution(challenge, &child1.routes) {
                offspring.push((child1, child1_cost));
            }
            if is_valid_solution(challenge, &child2.routes) {
                offspring.push((child2, child2_cost));
            }
        }

        population.extend(offspring);
        population.sort_by_key(|&(_, cost)| cost);
        population.truncate(2);
    }

    let final_solution = optimize_with_2_opt(
        Solution {
            routes: population[0].0.routes.clone(),
        },
        &challenge.distance_matrix,
    );

    return Some(final_solution);
}

fn is_valid_solution(challenge: &Challenge, routes: &Vec<Vec<usize>>) -> bool {
    let mut visited = vec![false; challenge.difficulty.num_nodes];
    visited[0] = true;

    for route in routes {
        if route.len() <= 2 || route[0] != 0 || route[route.len() - 1] != 0 {
            return false;
        }

        let mut capacity = challenge.max_capacity;

        for &node in &route[1..route.len() - 1] {
            if visited[node] {
                return false;
            }
            if challenge.demands[node] > capacity {
                return false;
            }
            visited[node] = true;
            capacity -= challenge.demands[node];
        }
    }

    if visited.iter().any(|&v| !v) {
        return false;
    }

    true
}

fn crossover_and_mutate(
    parent1: &Solution,
    parent2: &Solution,
    challenge: &Challenge,
    rng: &mut StdRng,
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

fn mutate(solution: Solution, distance_matrix: &Vec<Vec<i32>>, rng: &mut StdRng) -> Solution {
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

/*************************** */
/*************************** */
/*************************** */
/*************************** */

pub fn generate_clarke_wright_solution(challenge: &Challenge) -> Solution {
    let d = &challenge.distance_matrix;
    let c = challenge.max_capacity;
    let n = challenge.difficulty.num_nodes;

    // Clarke-Wright heuristic for node pairs based on their distances to depot
    // vs distance between each other
    let mut scores: Vec<(i32, usize, usize)> = Vec::with_capacity((n * (n - 1)) / 2);
    for i in 1..n {
        let d_i0 = d[i][0]; // Cache this value to avoid repeated lookups
        for j in (i + 1)..n {
            let score = d_i0 + d[0][j] - d[i][j];
            scores.push((score, i, j));
        }
    }
    scores.sort_unstable_by(|a, b| b.0.cmp(&a.0)); // Sort in descending order by score

    // Create a route for every node
    let mut routes: Vec<Option<Vec<usize>>> = (0..n).map(|i| Some(vec![i])).collect();
    routes[0] = None;
    let mut route_demands: Vec<i32> = challenge.demands.clone();

    // Iterate through node pairs, starting from greatest score
    for (s, i, j) in scores {
        // Stop if score is negative
        if s < 0 {
            break;
        }

        // Skip if joining the nodes is not possible
        if routes[i].is_none() || routes[j].is_none() {
            continue;
        }

        // Directly get the routes
        let (left_route, right_route) = (routes[i].as_ref().unwrap(), routes[j].as_ref().unwrap());

        // Cache indices and demands
        let (left_startnode, left_endnode) = (left_route[0], *left_route.last().unwrap());
        let (right_startnode, right_endnode) = (right_route[0], *right_route.last().unwrap());
        let merged_demand = route_demands[left_startnode] + route_demands[right_startnode];

        // Check constraints
        if left_startnode == right_startnode || merged_demand > c {
            continue;
        }

        // Merge routes
        let mut left_route = routes[i].take().unwrap();
        let mut right_route = routes[j].take().unwrap();
        routes[left_startnode] = None;
        routes[right_startnode] = None;
        routes[left_endnode] = None;
        routes[right_endnode] = None;

        // Reverse if needed
        if left_startnode == i {
            left_route.reverse();
        }
        if right_endnode == j {
            right_route.reverse();
        }

        // Create new route
        let mut new_route = left_route;
        new_route.extend(right_route);

        // Update routes and demands
        let (start, end) = (*new_route.first().unwrap(), *new_route.last().unwrap());
        routes[start] = Some(new_route.clone());
        routes[end] = Some(new_route);
        route_demands[start] = merged_demand;
        route_demands[end] = merged_demand;
    }

    let mut final_routes = Vec::new();

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

    optimize_with_2_opt(
        Solution {
            routes: final_routes,
        },
        &challenge.distance_matrix,
    )
}

pub fn generate_greedy_solution(challenge: &Challenge) -> Solution {
    let n = challenge.difficulty.num_nodes;
    let max_capacity = challenge.max_capacity;
    let distance_matrix = &challenge.distance_matrix;
    let demands = &challenge.demands;

    // Routes initialisées (une pour chaque véhicule, commençant au dépôt)
    let mut routes: Vec<Vec<usize>> = Vec::new();
    let mut visited = vec![false; n];
    visited[0] = true; // Le dépôt est toujours visité

    while visited.iter().any(|&v| !v) {
        let mut current_route = vec![0];
        let mut current_capacity = max_capacity;
        let mut current_node = 0;

        loop {
            // Trouver le prochain client qui minimise la distance ajoutée
            let mut next_node = None;
            let mut min_distance = i32::MAX;

            for i in 1..n {
                if !visited[i] && demands[i] <= current_capacity {
                    let dist = distance_matrix[current_node][i];
                    if dist < min_distance {
                        min_distance = dist;
                        next_node = Some(i);
                    }
                }
            }

            if let Some(next) = next_node {
                current_route.push(next);
                current_capacity -= demands[next];
                visited[next] = true;
                current_node = next;
            } else {
                break;
            }
        }

        // Retour au dépôt
        current_route.push(0);
        routes.push(current_route);
    }

    optimize_with_2_opt(Solution { routes: routes }, &challenge.distance_matrix)
}

pub fn generate_nearest_neighbor_solution(challenge: &Challenge) -> Solution {
    let d = &challenge.distance_matrix;
    let c = challenge.max_capacity;
    let n = challenge.difficulty.num_nodes;
    let demands = &challenge.demands;

    // Vecteur pour suivre les clients visités
    let mut visited = vec![false; n];
    visited[0] = true; // Le dépôt est toujours visité

    let mut routes = Vec::new(); // Stocke les différentes routes

    while visited.iter().any(|&v| !v) {
        // Tant qu'il reste des clients à visiter
        let mut route = vec![0]; // Chaque route commence au dépôt
        let mut current_node = 0; // Le dépôt est le premier nœud
        let mut capacity_remaining = c; // Capacité restante du véhicule

        while capacity_remaining > 0 {
            // Trouve le client non visité le plus proche avec une demande admissible
            let next_node = (1..n)
                .filter(|&i| !visited[i] && demands[i] <= capacity_remaining)
                .min_by_key(|&i| d[current_node][i]);

            if let Some(node) = next_node {
                // Si un client admissible est trouvé, le visiter
                route.push(node);
                visited[node] = true;
                capacity_remaining -= demands[node];
                current_node = node;
            } else {
                // Sinon, terminer la tournée
                break;
            }
        }

        route.push(0); // Retour au dépôt
        routes.push(route);
    }

    // Optimisation de la solution avec 2-opt
    optimize_with_2_opt(Solution { routes }, &challenge.distance_matrix)
}

pub fn generate_insertion_heuristic_solution(challenge: &Challenge) -> Solution {
    let d = &challenge.distance_matrix;
    let c = challenge.max_capacity;
    let n = challenge.difficulty.num_nodes;
    let demands = &challenge.demands;

    let mut routes: Vec<Vec<usize>> = vec![]; // Départ du dépôt
    let mut visited: Vec<bool> = vec![false; n];
    visited[0] = true;

    // Insertion heuristics - Insertion par coût minimal
    for _ in 1..n {
        let mut best_insertion: Option<(usize, usize, i32)> = None; // (route_index, position, cost)
        let mut best_node: Option<usize> = None;

        // Parcours des noeuds non visités
        for node in 1..n {
            if visited[node] {
                continue;
            }

            // Test sur toutes les tournées existantes
            for (r_idx, route) in routes.iter().enumerate() {
                for pos in 1..route.len() {
                    // Coût d'insertion du noeud dans la tournée courante à la position courante
                    let prev_node = route[pos - 1];
                    let next_node = route[pos];

                    let additional_cost =
                        d[prev_node][node] + d[node][next_node] - d[prev_node][next_node];

                    // On vérifie si la demande peut être satisfaite
                    let current_demand: i32 = route.iter().map(|&i| demands[i]).sum();
                    if current_demand + demands[node] <= c {
                        if best_insertion.is_none() || additional_cost < best_insertion.unwrap().2 {
                            best_insertion = Some((r_idx, pos, additional_cost));
                            best_node = Some(node);
                        }
                    }
                }
            }
        }

        // On effectue la meilleure insertion trouvée
        if let Some((r_idx, pos, _)) = best_insertion {
            let node = best_node.unwrap();
            routes[r_idx].insert(pos, node);
            visited[node] = true;
        } else {
            // Si aucune insertion possible, on commence une nouvelle tournée
            let node = (1..n).find(|&i| !visited[i]).unwrap();
            routes.push(vec![0, node, 0]); // Nouvelle tournée depuis le dépôt
            visited[node] = true;
        }
    }

    optimize_with_2_opt(Solution { routes: routes }, &challenge.distance_matrix)
}

fn generate_sweep_solution(challenge: &Challenge) -> Solution {
    let c = challenge.max_capacity;
    let n = challenge.difficulty.num_nodes;

    let mut rngs = RngArray::new(challenge.seeds);

    // Depot position is at node 0
    let depot_x = 250.0;
    let depot_y = 250.0;

    let mut node_positions: Vec<(f64, f64)> = (0..n)
        .map(|_| {
            (
                rngs.get_mut().gen::<f64>() * 500.0,
                rngs.get_mut().gen::<f64>() * 500.0,
            )
        })
        .collect();
    node_positions[0] = (250.0, 250.0);

    // Calculate polar angle from depot for each customer (excluding the depot itself)
    let mut nodes_with_angles: Vec<(usize, f64)> = (1..n)
        .map(|i| {
            let (x, y) = node_positions[i];
            let angle = (y - depot_y).atan2(x - depot_x);
            (i, if angle < 0.0 { angle + 2.0 * PI } else { angle })
        })
        .collect();

    // Sort nodes by polar angle
    nodes_with_angles.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    // Sweep algorithm: create routes by adding nodes until the capacity is exceeded
    let mut routes = Vec::new();
    let mut current_route = vec![0];
    let mut current_capacity = 0;

    for (node, _) in nodes_with_angles {
        let demand = challenge.demands[node];
        if current_capacity + demand <= c {
            current_route.push(node);
            current_capacity += demand;
        } else {
            // Complete the current route and start a new one
            current_route.push(0);
            routes.push(current_route);
            current_route = vec![0, node];
            current_capacity = demand;
        }
    }

    // Add the last route if it has nodes
    if current_route.len() > 1 {
        current_route.push(0);
        routes.push(current_route);
    }

    optimize_with_2_opt(Solution { routes: routes }, &challenge.distance_matrix)
}
