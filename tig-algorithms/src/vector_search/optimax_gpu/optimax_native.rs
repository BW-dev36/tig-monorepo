/*!
Copyright 2024 bw-dev36

Licensed under the TIG Inbound Game License v1.0 or (at your option) any later
version (the "License"); you may not use this file except in compliance with the
License. You may obtain a copy of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/

use anyhow::Ok;
//use std::time::Instant;
use tig_challenges::vector_search::*;



#[repr(C)]
pub struct VSOChallenge {
    vector_database: *const f32,
    vector_database_len: usize,
    vector_sizes: *const usize,
    num_vectors: usize,

    query_vectors: *const f32,
    query_vectors_len: usize,
    query_sizes: *const usize,
    num_queries: usize,

    max_distance: f32,
    difficulty: u32,
}

#[repr(C)]
pub struct VSOSolution {
    indexes: *mut usize,
    len: usize,
}

extern "C" {
    fn solve_optimax_cpp(challenge: *const VSOChallenge, solution: *mut VSOSolution);
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {

    let vector_database: Vec<f32> = challenge.vector_database.iter().flatten().copied().collect();
    let vector_sizes: Vec<usize> = challenge.vector_database.iter().map(|v| v.len()).collect();

    let query_vectors: Vec<f32> = challenge.query_vectors.iter().flatten().copied().collect();
    let query_sizes: Vec<usize> = challenge.query_vectors.iter().map(|v| v.len()).collect();

    let vs_challenge = VSOChallenge {
        vector_database: vector_database.as_ptr(),
        vector_database_len: vector_database.len(),
        vector_sizes: vector_sizes.as_ptr(),
        num_vectors: vector_sizes.len(),

        query_vectors: query_vectors.as_ptr(),
        query_vectors_len: query_vectors.len(),
        query_sizes: query_sizes.as_ptr(),
        num_queries: query_sizes.len(),

        max_distance: challenge.max_distance,
        difficulty: challenge.difficulty.better_than_baseline,
    };

    let mut indexes: Vec<usize> = vec![0; challenge.query_vectors.len()];

    let mut vs_solution = VSOSolution {
        indexes: indexes.as_mut_ptr(),
        len: 0,
    };

    unsafe {
        solve_optimax_cpp(&vs_challenge, &mut vs_solution);
    }

    if vs_solution.len == 0 {
        Ok(None)
    } else {
        indexes.truncate(vs_solution.len);
        Ok(Some(Solution { indexes }))
    }
}
