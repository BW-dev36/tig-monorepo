use super::query_data;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::error::Error;



pub async fn select_algorithms_to_run(
    player_id: &str,
    _duration: u32,
    algo_selection: &HashMap<String, String>
) -> Result<(HashMap<String, (String, u32)>, u32), Box<dyn Error>> {
    // Crée un map par défaut pour algo_map
    let mut algo_map: HashMap<String, (String, u32)> = HashMap::new();
    algo_map.insert("c001".to_string(), ("satisfiability".to_string(), 22000));
    algo_map.insert("c002".to_string(), ("vehicle_routing".to_string(), 22000));
    algo_map.insert("c003".to_string(), ("knapsack".to_string(), 22000));
    algo_map.insert("c004".to_string(), ("vector_search".to_string(), 22000));

    // Récupérer les solutions à partir de l'ID du joueur
    let solutions: HashMap<String, u32> = fetch_solutions(player_id).await?;

    
    let opt_algos_to_run: Option<(String, u32, u32)> = determine_algorithms_to_run(&solutions, &algo_map);
    
    let config: Option<HashMap<String, (String, u32)>> = generate_algo_map(&opt_algos_to_run, algo_selection);
    
    if config.is_none() {
        
        println!("Warning no algo !!!!");
        
        let mut algos_with_duration = HashMap::new();
        for (challenge_id, algorithm_id) in algo_selection
        {
            let (_, duration) = algo_map.get(challenge_id).cloned().unwrap();
            algos_with_duration.insert(challenge_id.clone(), (algorithm_id.clone(), duration));
        };
        
        Ok((algos_with_duration.clone(), 0))
    } else {
        
        match opt_algos_to_run {
            Some((_, solution_count, _)) => Ok((config.unwrap(), solution_count)),
            None => 
            {
                let mut algos_with_duration = HashMap::new();
                for (challenge_id, algorithm_id) in algo_selection
                {
                    let (_, duration) = algo_map.get(challenge_id).cloned().unwrap();
                    algos_with_duration.insert(challenge_id.clone(), (algorithm_id.clone(), duration));
                };
                
                Ok((algos_with_duration.clone(), 0))
            } 
        }
    }
}

async fn fetch_solutions(player_id: &str) -> Result<HashMap<String, u32>, Box<dyn Error>> {
    let mut solutions: HashMap<String, u32> = HashMap::new();

    let query_data = query_data::execute().await?;
    for challenge in query_data.algorithms_by_challenge {
        for algo in challenge.1 {
            if let Some(block_data) = algo.block_data {
                if let Some(num_qualifiers) = block_data.num_qualifiers_by_player().get(player_id) {
                    let entry = solutions
                        .entry(algo.details.challenge_id.clone())
                        .or_insert(0);
                    *entry += num_qualifiers;
                }
            }
        }
    }

    Ok(solutions)
}

fn determine_algorithms_to_run(
    solutions: &HashMap<String, u32>,
    map: &HashMap<String, (String, u32)>,
) -> Option<(String, u32, u32)> {
    // Si le HashMap des solutions est vide et que le map n'est pas vide, retourner None
    if map.is_empty() {
        return None;
    }

    // Créer un nouveau HashMap pour stocker les solutions avec valeurs par défaut
    let mut completed_solutions: HashMap<String, (String, u32, u32)> = HashMap::new();

    // Remplir completed_solutions avec tous les algorithmes de map, utilisant 0 comme valeur par défaut si absent dans solutions
    for (algo, (exec_name, duration)) in map.iter() {
        let solution_count = solutions.get(algo).cloned().unwrap_or(0);
        completed_solutions.insert(algo.clone(), (exec_name.clone(), solution_count, *duration));
    }

    //println!("Solutions by algo : {:#?}", completed_solutions);

    // Trouver le nombre minimum de solutions trouvées parmi tous les algorithmes
    let min_solutions = completed_solutions
        .values()
        .map(|(_, solution_count, _)| solution_count)
        .min()
        .unwrap_or(&0);

    // Rechercher les algorithmes avec le nombre minimum de solutions
    let least_solution_algorithms: Vec<&String> = completed_solutions
        .iter()
        .filter(|&(_, &(_, solution_count, _))| solution_count == *min_solutions)
        .map(|(algorithm, _)| algorithm)
        .collect();

    // Vérifier que le vecteur n'est pas vide
    if least_solution_algorithms.is_empty() {
        return None;
    }

    // Obtenir le premier algorithme avec le moins de solutions
    let algo_name = least_solution_algorithms[0];

    // Obtenir les détails de l'algorithme à partir de completed_solutions
    if let Some((exec_name, solution_count, duration)) = completed_solutions.get(algo_name) {
        Some((exec_name.clone(), *solution_count, *duration))
    } else {
        None
    }
}

fn generate_algo_map(
    algos_to_run: &Option<(String, u32, u32)>,
    algo_selection: &HashMap<String, String>,
) -> Option<HashMap<String, (String, u32)>> {
    let mut config = HashMap::new();
    if algos_to_run.is_none()
    {
            println!("No algo found with lowest solutions");
            return None;
    }
    if let Some((ref algo_name, _, duration)) = algos_to_run {
        for (name, exec_name) in algo_selection {
            //println!("algo_name {} vs {}", algo_name, name);
            if algo_name == name {
                config.insert(name.clone(), (exec_name.clone(), duration.clone()));
            }
        }
    }

    Some(config)
}

fn calculate_duration(
    config: &HashMap<String, String>,
    algo_map: &HashMap<String, String>,
    original_duration: u32,
    new_duration: u32,
) -> u32 {
    let mut duration = original_duration;
    if config.len() != algo_map.len() {
        duration = new_duration;
    }
    duration
}
