use crate::{ChallengeTrait, DifficultyTrait, RngArray, SolutionTrait};
use anyhow::{anyhow, Ok, Result};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Map, Value};

#[cfg(feature = "cuda")]
use crate::CudaKernel;
#[cfg(feature = "cuda")]
use cudarc::driver::*;
#[cfg(feature = "cuda")]
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Difficulty {
    pub num_queries: u32,
    pub better_than_baseline: u32,
}

impl DifficultyTrait<2> for Difficulty {
    fn from_arr(arr: &[i32; 2]) -> Self {
        Self {
            num_queries: arr[0] as u32,
            better_than_baseline: arr[1] as u32,
        }
    }

    fn to_arr(&self) -> [i32; 2] {
        [self.num_queries as i32, self.better_than_baseline as i32]
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(PartialEq)]
pub struct Solution {
    pub indexes: Vec<usize>,
}

impl SolutionTrait for Solution {}

impl TryFrom<Map<String, Value>> for Solution {
    type Error = serde_json::Error;

    fn try_from(v: Map<String, Value>) -> Result<Self, Self::Error> {
        from_value(Value::Object(v))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    pub seeds: [u64; 8],
    pub difficulty: Difficulty,
    pub vector_database: Vec<Vec<f32>>,
    pub query_vectors: Vec<Vec<f32>>,
    pub max_distance: f32,
}

pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(&x1, &x2)| (x1 - x2) * (x1 - x2))
        .sum::<f32>()
        .sqrt()
}

// TIG dev bounty available for a GPU optimisation for instance generation!
#[cfg(feature = "cuda")]
pub const KERNEL: Option<CudaKernel> = None;

impl ChallengeTrait<Solution, Difficulty, 2> for Challenge {
    #[cfg(feature = "cuda")]
    fn cuda_generate_instance(
        seeds: [u64; 8],
        difficulty: &Difficulty,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> Result<Self> {
        // TIG dev bounty available for a GPU optimisation for instance generation!
        Self::generate_instance(seeds, difficulty)
    }

    //#[cfg(feature = "native")]
    // fn generate_instance_native(seeds: [u64; 8], difficulty: &Difficulty) -> Result<Self> {
    //     Ok(self);
    // }

    fn generate_instance(seeds: [u64; 8], difficulty: &Difficulty) -> Result<Self> {
        let mut rngs = RngArray::new(seeds);
        let uniform = Uniform::from(0.0..1.0);
        let search_vectors = (0..100000)
            .map(|_| (0..250).map(|_| uniform.sample(rngs.get_mut())).collect())
            .collect();
        let query_vectors = (0..difficulty.num_queries)
            .map(|_| (0..250).map(|_| uniform.sample(rngs.get_mut())).collect())
            .collect();
        let max_distance = 6.0 - (difficulty.better_than_baseline as f32) / 1000.0;

        Ok(Self {
            seeds,
            difficulty: difficulty.clone(),
            vector_database: search_vectors,
            query_vectors,
            max_distance,
        })
    }

    fn verify_solution(&self, solution: &Solution) -> Result<()> {
        if solution.indexes.len() != self.difficulty.num_queries as usize {
            return Err(anyhow!(
                "Invalid number of indexes. Expected: {}, Actual: {}",
                self.difficulty.num_queries,
                solution.indexes.len()
            ));
        }

        let mut dists = Vec::new();
        for (query, &search_index) in self.query_vectors.iter().zip(solution.indexes.iter()) {
            if search_index >= self.vector_database.len() {
                return Err(anyhow!(
                    "Invalid index. Expected: less than {}, Actual: {}",
                    self.vector_database.len(),
                    search_index
                ));
            }
            let search = &self.vector_database[search_index];
            dists.push(euclidean_distance(query, search));
        }
        let avg_dist = dists.iter().sum::<f32>() / dists.len() as f32;
        if avg_dist > self.max_distance {
            return Err(anyhow!(
                "Average query vector distance is '{}'. Max dist: '{}'",
                avg_dist,
                self.max_distance
            ));
        }
        Ok(())
    }
}

pub fn convert_vsochallenge_to_challenge(vso_challenge: *const tig_native::vector_search::VSOChallenge) -> Challenge {
    unsafe {
        // Déréférencer le pointeur
        let challenge_ref = &*vso_challenge;

        // Conversion du tableau de seeds
        let seeds = challenge_ref.seeds;

        // Conversion de difficulty
        let difficulty = Difficulty {
            num_queries: challenge_ref.difficulty.num_queries,
            better_than_baseline: challenge_ref.difficulty.better_than_baseline,
        };

        // Conversion de vector_database (pointeurs bruts vers Vec<Vec<f32>>)
        let vector_database = (0..challenge_ref.vector_database_size)
            .map(|i| {
                let vec_ptr = challenge_ref.vector_database.add(i);
                let vec = std::slice::from_raw_parts(*vec_ptr, 250); // Taille fixe de 250
                vec.to_vec() // Convertir le slice en Vec<f32>
            })
            .collect::<Vec<Vec<f32>>>();

        // Conversion de query_vectors (pointeurs bruts vers Vec<Vec<f32>>)
        let query_vectors = (0..challenge_ref.query_vectors_size)
            .map(|i| {
                let query_ptr = challenge_ref.query_vectors.add(i);
                let vec = std::slice::from_raw_parts(*query_ptr, 250); // Taille fixe de 250
                vec.to_vec() // Convertir le slice en Vec<f32>
            })
            .collect::<Vec<Vec<f32>>>();

        // Conversion de max_distance
        let max_distance = challenge_ref.max_distance;

        // Retourner l'objet Challenge
        Challenge {
            seeds,
            difficulty,
            vector_database,
            query_vectors,
            max_distance,
        }
    }
}