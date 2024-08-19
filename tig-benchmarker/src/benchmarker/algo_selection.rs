use serde_json::{self, Value};
use std::cmp;
use std::collections::HashMap;
use std::error::Error;

pub async fn select_algorithms_to_run(
    player_id: String,
    algo_selection: &HashMap<String, String>,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut algo_map = HashMap::new();
    algo_map.insert("c001".to_string(), "satisfiability".to_string());
    algo_map.insert("c002".to_string(), "vehicle_routing".to_string());
    algo_map.insert("c003".to_string(), "knapsack".to_string());
    algo_map.insert("c004".to_string(), "vector_search".to_string());

    if let Some(block_id) = fetch_block_id().await? {
        let solutions = fetch_solutions(player_id, &block_id).await?;
        let algos_to_run = determine_algorithms_to_run(&solutions, &algo_map, 1.5);
        let config = generate_algo_map(&algos_to_run, algo_selection);

        if config.is_empty() {
            Ok(algo_selection.clone())
        } else {
            Ok(config)
        }
    } else {
        Ok(algo_selection.clone())
    }
}

async fn fetch_block_id() -> Result<Option<String>, Box<dyn Error>> {
    let block_response: Value = reqwest::get("https://mainnet-api.tig.foundation/get-block")
        .await?
        .json()
        .await?;
    Ok(block_response["block"]["id"].as_str().map(String::from))
}

async fn fetch_solutions(
    player_id: String,
    block_id: &str,
) -> Result<HashMap<String, i32>, Box<dyn Error>> {
    let mut solutions: HashMap<String, i32> = HashMap::new();

    let url = format!(
        "https://mainnet-api.tig.foundation/get-algorithms?block_id={}",
        block_id
    );
    let response: Value = reqwest::get(&url).await?.json().await?;

    if let Some(algorithms) = response["algorithms"].as_array() {
        for algo in algorithms {
            if let Some(challenge_id) = algo["details"]["challenge_id"].as_str() {
                if let Some(num_qualifiers) =
                    algo["block_data"]["num_qualifiers_by_player"][player_id.clone()].as_i64()
                {
                    let entry = solutions.entry(challenge_id.to_string()).or_insert(0);
                    *entry += num_qualifiers as i32;
                }
            }
        }
    }

    Ok(solutions)
}

fn determine_algorithms_to_run(
    solutions: &HashMap<String, i32>,
    algo_map: &HashMap<String, String>,
    cutoff_multiplier: f64,
) -> Vec<String> {
    let least_solutions = *solutions.values().min().unwrap_or(&0);

    let cutoff = cmp::max(
        3,
        ((least_solutions as f64) * cutoff_multiplier).ceil() as i32,
    );

    let mut algos_to_run = vec![];
    for (challenge, &count) in solutions {
        if count < cutoff {
            if let Some(algo) = algo_map.get(challenge) {
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
