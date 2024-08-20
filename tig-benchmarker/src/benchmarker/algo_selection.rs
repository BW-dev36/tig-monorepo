use super::query_data;
use serde_json::{self, Value};
use std::cmp;
use std::collections::HashMap;
use std::error::Error;

pub async fn select_algorithms_to_run(
    player_id: &str,
    algo_selection: &HashMap<String, String>,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut algo_map = HashMap::new();
    algo_map.insert("c001".to_string(), "satisfiability".to_string());
    algo_map.insert("c002".to_string(), "vehicle_routing".to_string());
    algo_map.insert("c003".to_string(), "knapsack".to_string());
    algo_map.insert("c004".to_string(), "vector_search".to_string());

    let solutions = fetch_solutions(player_id).await?;
    let algos_to_run = determine_algorithms_to_run(&solutions, &algo_map, 1.5, 10.0);
    let config = generate_algo_map(&algos_to_run, algo_selection);

    if config.is_empty() {
        Ok(algo_selection.clone())
    } else {
        Ok(config)
    }
}

async fn fetch_block_id() -> Result<Option<String>, Box<dyn Error>> {
    let block_response: Value = reqwest::get("https://mainnet-api.tig.foundation/get-block")
        .await?
        .json()
        .await?;
    Ok(block_response["block"]["id"].as_str().map(String::from))
}

async fn fetch_solutions(player_id: &str) -> Result<HashMap<String, u32>, Box<dyn Error>> {
    let mut solutions: HashMap<String, u32> = HashMap::new();

    let query_data = query_data::execute().await?;
    for challenge in query_data.algorithms_by_challenge {
        for algo in challenge.1 {
            if let Some(num_qualifiers) =
                algo.block_data().num_qualifiers_by_player().get(player_id)
            {
                let entry = solutions
                    .entry(algo.details.challenge_id.clone())
                    .or_insert(0);
                *entry += num_qualifiers;
            }
        }
    }

    Ok(solutions)
}

fn determine_algorithms_to_run(
    solutions: &HashMap<String, u32>,
    algo_map: &HashMap<String, String>,
    cutoff_multiplier: f32,
    tolerance_percentage: f32,
) -> Vec<String> {
    let total_solutions: f32 = solutions.values().sum::<u32>() as f32;
    let num_challenges = algo_map.len() as f32;

    if num_challenges == 0.0 {
        return Vec::new();
    }

    let average_solutions = total_solutions / num_challenges;
    let threshold = (average_solutions * (1.0 - tolerance_percentage / 100.0)) as u32;

    let mut algos_to_run = Vec::new();
    for (challenge, algo) in algo_map {
        let count = solutions.get(challenge).cloned().unwrap_or(0);
        if count < threshold {
            algos_to_run.push(algo.clone());
        }
    }

    if algos_to_run.is_empty() {
        let least_solutions = *solutions.values().min().unwrap_or(&0);
        let cutoff = cmp::max(
            3,
            ((least_solutions as f32) * cutoff_multiplier).ceil() as u32,
        );

        for (challenge, algo) in algo_map {
            let count = solutions.get(challenge).cloned().unwrap_or(0);
            if count < cutoff {
                algos_to_run.push(algo.clone());
            }
        }
    }

    algos_to_run
}

fn generate_algo_map(
    algos_to_run: &[String],
    algo_selection: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut config = HashMap::new();

    for (algo_name, exec_name) in algo_selection {
        if algos_to_run.contains(algo_name) {
            config.insert(algo_name.clone(), exec_name.clone());
        }
    }

    config
}
