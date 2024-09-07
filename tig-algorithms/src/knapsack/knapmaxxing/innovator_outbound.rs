/*!
Copyright 2024 bw-dev36

Licensed under the TIG Innovator Outbound Game License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/
use tig_challenges::knapsack::*;


// use ffi::{knapsack_solver, Challenge as FfiChallenge};

use crate::knapsack::knapmaxxing::ffi;
use ffi::{knapsack_knapmaxxing_solver, Challenge as FfiChallenge};

pub fn solve_challenge(challenge: &tig_challenges::knapsack::Challenge) -> anyhow::Result<Option<tig_challenges::knapsack::Solution>> {
    let values: Vec<u32> = challenge.values.iter().map(|&v| v as u32).collect();
    let weights: Vec<u32> = challenge.weights.iter().map(|&w| w as u32).collect();

    let c_challenge = FfiChallenge {
        max_weight: challenge.max_weight as u32,
        min_value: challenge.min_value as u32,
        num_items: challenge.difficulty.num_items as u32,
        values: values.as_ptr(),
        weights: weights.as_ptr(),
    };

    // Appel de la fonction C
    if let Some(solution_indices) = knapsack_knapmaxxing_solver(&c_challenge) {
        Ok(Some(Solution { items: solution_indices }))
    } else {
        Ok(None)
    }
}

#[cfg(feature = "cuda")]
mod gpu_optimisation {
    use super::*;
    use cudarc::driver::*;
    use std::{collections::HashMap, sync::Arc};
    use tig_challenges::CudaKernel;

    // set KERNEL to None if algorithm only has a CPU implementation
    pub const KERNEL: Option<CudaKernel> = None;

    // Important! your GPU and CPU version of the algorithm should return the same result
    pub fn cuda_solve_challenge(
        challenge: &Challenge,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<Option<Solution>> {
        solve_challenge(challenge)
    }
}
#[cfg(feature = "cuda")]
pub use gpu_optimisation::{cuda_solve_challenge, KERNEL};
