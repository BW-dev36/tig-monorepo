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
use tig_native::vector_search::*;

pub fn solve_challenge_native(challenge:*mut VSOChallenge) -> anyhow::Result<Option<tig_challenges::vector_search::Solution>> {
    // Allocation de la solution en C++
    let mut solution_indexes = vec![0_usize; unsafe { &*challenge }.difficulty.num_queries as usize];
    let mut vs_solution = VSOSolution {
        indexes: solution_indexes.as_mut_ptr(),
        len: 0,
    };

    // Appel de la fonction C++ optimisée
    unsafe {
        solve_optimax_cpp(challenge, &mut vs_solution);
    }

    // Traiter la solution retournée par `solve_optimax_cpp`
    if vs_solution.len == 0 {
        Ok(None)
    } else {
        solution_indexes.truncate(vs_solution.len);
        Ok(Some(Solution { indexes: solution_indexes }))
    }
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    // Conversion de `challenge.difficulty` en `VSODifficulty`
    let vs_difficulty = VSODifficulty {
        num_queries: challenge.difficulty.num_queries,
        better_than_baseline: challenge.difficulty.better_than_baseline,
    };

    // Allocation dynamique des bases de données de vecteurs dans un format compatible avec le C++ (pointeurs sur tableaux)
    let vector_database: Vec<*mut f32> = challenge
        .vector_database
        .iter()
        .map(|vec| vec.as_ptr() as *mut f32)
        .collect();

    let query_vectors: Vec<*mut f32> = challenge
        .query_vectors
        .iter()
        .map(|vec| vec.as_ptr() as *mut f32)
        .collect();

    // Préparation de l'objet VSOChallenge
    let vs_challenge = VSOChallenge {
        seeds: challenge.seeds,
        difficulty: vs_difficulty,
        vector_database: vector_database.as_ptr() as *mut *mut f32,
        query_vectors: query_vectors.as_ptr() as *mut *mut f32,
        vector_database_size: vector_database.len(),
        query_vectors_size: query_vectors.len(),
        max_distance: challenge.max_distance,
    };

    // Allocation de la solution en C++
    let mut solution_indexes = vec![0_usize; challenge.difficulty.num_queries as usize];
    let mut vs_solution = VSOSolution {
        indexes: solution_indexes.as_mut_ptr(),
        len: 0,
    };

    // Appel de la fonction C++ optimisée
    unsafe {
        solve_optimax_cpp(&vs_challenge, &mut vs_solution);
    }

    // Traiter la solution retournée par `solve_optimax_cpp`
    if vs_solution.len == 0 {
        Ok(None)
    } else {
        solution_indexes.truncate(vs_solution.len);
        Ok(Some(Solution { indexes: solution_indexes }))
    }
}


#[cfg(feature = "cuda")]
mod gpu_optimisation {
    use crate::vector_search::optimax_gpu::benchmarker_outbound;

    use super::*;
    use cudarc::driver::*;
    use std::{collections::HashMap, sync::Arc};
    use tig_challenges::CudaKernel;
    pub const KERNEL: Option<CudaKernel> = Some(CudaKernel {
        src: r#"
        
        extern "C" __global__ void filter_vectors(float* query_mean, float* vectors, float* distances, int num_vectors, int num_dimensions) {
            int idx = blockIdx.x * blockDim.x + threadIdx.x;
            if (idx < num_vectors) {
                float dist = 0.0;
                for (int d = 0; d < num_dimensions; ++d) {
                    float diff = query_mean[d] - vectors[idx * num_dimensions + d];
                    dist += diff * diff;
                }
                distances[idx] = dist;
            }
        }
        
        "#,

        funcs: &["filter_vectors"],
    });

    pub fn cuda_solve_challenge(
        challenge: &Challenge,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> anyhow::Result<Option<Solution>> {
        return benchmarker_outbound::solve_challenge_outbound(challenge);
    }

}
#[cfg(feature = "cuda")]
pub use gpu_optimisation::{cuda_solve_challenge, KERNEL};
