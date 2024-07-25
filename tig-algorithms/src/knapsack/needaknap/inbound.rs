/*!
Copyright 2024 Daniel Shaver

Licensed under the TIG Inbound Game License v1.0 or (at your option) any later
version (the "License"); you may not use this file except in compliance with the
License. You may obtain a copy of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/


use tig_challenges::knapsack::*;

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let max_weight = challenge.max_weight as usize;
    let min_value = challenge.min_value as usize;
    let num_items = challenge.difficulty.num_items;
    let max_weight_plus_one = max_weight + 1;

    let mut weights = Vec::with_capacity(num_items);
    let mut values = Vec::with_capacity(num_items);
    for (&weight, &value) in challenge.weights.iter().zip(challenge.values.iter()) {
        weights.push(weight as usize);
        values.push(value as usize);
    }

    let mut sorted_items: Vec<(usize, f64)> = (0..num_items)
        .map(|i| (i, values[i] as f64 / weights[i] as f64))
        .collect();
    sorted_items.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let upper_bound = calculate_upper_bound(&sorted_items, &weights, &values, max_weight);
    if upper_bound < min_value {
        return Ok(None);
    }

    let num_states = (num_items + 1) * max_weight_plus_one;
    let mut dp = vec![0; num_states];

    for i in 1..=num_items {
        let (item_index, _) = sorted_items[i - 1];
        let item_weight = weights[item_index];
        let item_value = values[item_index];
        let i_minus_one_times_max_weight_plus_one = (i - 1) * max_weight_plus_one;
        let i_times_max_weight_plus_one = i * max_weight_plus_one;

        for w in (item_weight..=max_weight).rev() {
            let prev_state = i_minus_one_times_max_weight_plus_one + w;
            let curr_state = i_times_max_weight_plus_one + w;
            dp[curr_state] = dp[prev_state].max(dp[prev_state - item_weight] + item_value);
        }
    }

    let solution = backtrack(&dp, &sorted_items, &weights, &values, num_items, max_weight, min_value, max_weight_plus_one);
    Ok(solution)
}

fn calculate_upper_bound(sorted_items: &[(usize, f64)], weights: &[usize], values: &[usize], max_weight: usize) -> usize {
    let mut upper_bound = 0;
    let mut remaining_weight = max_weight;
    for &(item_index, ratio) in sorted_items {
        let item_weight = weights[item_index];
        let item_value = values[item_index];
        if item_weight <= remaining_weight {
            upper_bound += item_value;
            remaining_weight -= item_weight;
        } else {
            upper_bound += (ratio * remaining_weight as f64).floor() as usize;
            break;
        }
    }
    upper_bound
}

fn backtrack(dp: &[usize], sorted_items: &[(usize, f64)], weights: &[usize], values: &[usize], num_items: usize, max_weight: usize, min_value: usize, max_weight_plus_one: usize) -> Option<Solution> {
    let mut items = Vec::with_capacity(num_items);
    let mut i = num_items;
    let mut w = max_weight;
    let mut total_value = 0;

    while i > 0 && total_value < min_value {
        let (item_index, _) = sorted_items[i - 1];
        let item_weight = weights[item_index];
        let item_value = values[item_index];
        let prev_state = (i - 1) * max_weight_plus_one + w;
        let curr_state = i * max_weight_plus_one + w;

        if dp[curr_state] == dp[prev_state] {
            i -= 1;
            continue;
        }

        items.push(item_index);
        w -= item_weight;
        total_value += item_value;
        i -= 1;
    }

    if total_value >= min_value {
        Some(Solution { items })
    } else {
        None
    }
}