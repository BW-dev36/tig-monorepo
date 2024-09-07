/*!
Copyright 2024 Louis Silva

Licensed under the TIG Benchmarker Outbound Game License v1.0 (the "License"); you
may not use this file except in compliance with the License. You may obtain a copy
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
 */

use anyhow::Result;

use tig_challenges::vector_search::*;
#[repr(C)]
pub struct VSChallenge {
    vector_database: *const f32, // Pointeur vers les données des vecteurs dans la base
    vector_database_len: usize,  // Nombre total d'éléments dans la base
    vector_sizes: *const usize,  // Taille de chaque vecteur dans la base
    num_vectors: usize,          // Nombre de vecteurs dans la base

    query_vectors: *const f32, // Pointeur vers les données des vecteurs de requête
    query_vectors_len: usize,  // Nombre total d'éléments de requête
    query_sizes: *const usize, // Taille de chaque vecteur de requête
    num_queries: usize,        // Nombre de vecteurs de requête

    max_distance: f32,
}

#[repr(C)]
pub struct VSSolution {
    indexes: *mut usize, // Pointeur vers les indices des solutions trouvées
    len: usize,          // Nombre d'indices trouvés
}

extern "C" {
    fn solve_bacalhau_v1_cpp(challenge: *const VSChallenge, solution: *mut VSSolution);
}

pub fn solve_challenge(challenge: &Challenge) -> Result<Option<Solution>> {
    // Préparer les données pour l'interface C
    let vector_database: Vec<f32> = challenge
        .vector_database
        .iter()
        .flatten()
        .copied()
        .collect();
    let vector_sizes: Vec<usize> = challenge.vector_database.iter().map(|v| v.len()).collect();

    let query_vectors: Vec<f32> = challenge.query_vectors.iter().flatten().copied().collect();
    let query_sizes: Vec<usize> = challenge.query_vectors.iter().map(|v| v.len()).collect();

    let vs_challenge = VSChallenge {
        vector_database: vector_database.as_ptr(),
        vector_database_len: vector_database.len(),
        vector_sizes: vector_sizes.as_ptr(),
        num_vectors: vector_sizes.len(),

        query_vectors: query_vectors.as_ptr(),
        query_vectors_len: query_vectors.len(),
        query_sizes: query_sizes.as_ptr(),
        num_queries: query_sizes.len(),

        max_distance: challenge.max_distance,
    };

    // Préparer un vecteur pour les résultats
    let mut indexes: Vec<usize> = vec![0; challenge.query_vectors.len()];

    let mut vs_solution = VSSolution {
        indexes: indexes.as_mut_ptr(),
        len: 0,
    };

    // Appeler la fonction C++
    unsafe {
        solve_bacalhau_v1_cpp(&vs_challenge, &mut vs_solution);
    }

    if vs_solution.len == 0 {
        Ok(None)
    } else {
        indexes.truncate(vs_solution.len);
        Ok(Some(Solution { indexes }))
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
