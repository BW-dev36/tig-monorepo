/*!
Copyright 2024 AllFather

Licensed under the TIG Benchmarker Outbound Game License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https:

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use std::slice;

#[repr(C)]
pub struct CChallenge {
    pub seed: u64,
    pub num_variables: i32,
    pub clauses: *mut i32,
    pub clause_lengths: *mut i32,
    pub num_clauses: i32,
}

#[repr(C)]
pub struct CSolution {
    pub variables: *mut bool,
    pub num_variables: i32,
}

#[link(name = "rust_lib", kind = "static")]
extern "C" {
    fn solve_sprint_sat_v2_cpp(challenge: *const CChallenge, solution: *mut CSolution);
}


use tig_challenges::satisfiability::{Challenge, Solution};

fn challenge_to_c(challenge: &Challenge) -> (CChallenge, Vec<i32>, Vec<i32>) {
    let mut flat_clauses: Vec<i32> = Vec::new();
    let mut clause_lengths: Vec<i32> = Vec::new();

    for clause in &challenge.clauses {
        clause_lengths.push(clause.len() as i32);
        flat_clauses.extend(clause);
    }

    let c_challenge = CChallenge {
        seed: challenge.seeds[0],
        num_variables: challenge.difficulty.num_variables as i32, 
        clauses: flat_clauses.as_mut_ptr(),
        clause_lengths: clause_lengths.as_mut_ptr(),
        num_clauses: challenge.clauses.len() as i32,
    };

    (c_challenge, flat_clauses, clause_lengths)
}

fn solution_from_c(c_solution: &CSolution) -> Solution {
    let variables = unsafe {
        slice::from_raw_parts(c_solution.variables, c_solution.num_variables as usize).to_vec()
    };
    Solution { variables }
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let mut solution_variables = vec![false; challenge.clauses.len()];
    let mut c_solution = CSolution {
        variables: solution_variables.as_mut_ptr(),
        num_variables: challenge.difficulty.num_variables as i32,
    };

    let (c_challenge, _flat_clauses, _clause_lengths) = challenge_to_c(challenge);
    
    unsafe {
        solve_sprint_sat_v2_cpp(&c_challenge, &mut c_solution);
    }

    
    let solution = solution_from_c(&c_solution);

    
    if solution.variables.iter().all(|&x| !x) {
        Ok(None)
    } else {
        Ok(Some(solution))
    }
}

#[cfg(feature = "cuda")]
mod gpu_optimisation {
    use super::*;
    use cudarc::driver::*;
    use std::{collections::HashMap, sync::Arc};
    use tig_challenges::CudaKernel;

    
    pub const KERNEL: Option<CudaKernel> = None;

    
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
