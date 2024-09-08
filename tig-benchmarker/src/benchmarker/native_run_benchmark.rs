use super::{Job, NonceIterator};
use crate::utils;
use std::sync::Arc;
use tig_native::vector_search::*;
use tig_algorithms::{c001, c002, c003, c004};
use tig_challenges::ChallengeTrait;
use tig_worker::{compute_solution, verify_solution};
use tig_worker::{native_compute_solution, native_verify_solution};
use tokio::{spawn, sync::Mutex, task::yield_now};
use utils::time;
use anyhow::{anyhow, Result};

pub async fn execute(nonce_iterators: Vec<Arc<Mutex<NonceIterator>>>, job: &Job, wasm: &Vec<u8>) {
    println!("Running WOOOOOORRRRRLLD\n");
    for nonce_iterator in nonce_iterators {
        let job = job.clone();
        let wasm = wasm.clone();
        spawn(async move {
            let mut last_yield = time();
            loop {
                let now = time();
                if now >= job.timestamps.end {
                    break;
                }
                match { nonce_iterator.lock().await.next() } {
                    None => break,
                    Some(nonce) => {
                        if now - last_yield > 25 {
                            yield_now().await;
                            last_yield = now;
                        }
                        let seeds = job.settings.calc_seeds(nonce);
                        let skip = match job.settings.challenge_id.as_str() {
                            "c001" => {
                                type SolveChallengeFn =
                                    fn(
                                        &tig_challenges::c001::Challenge,
                                    )
                                        -> anyhow::Result<Option<tig_challenges::c001::Solution>>;
                                match match job.settings.algorithm_id.as_str() {
                                    #[cfg(feature = "c001_a001")]
                                    "c001_a001" => Some(c001::c001_a001::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c001_a005")]
                                    "c001_a005" => Some(c001::c001_a005::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c001_a011")]
                                    "c001_a011" => Some(c001::c001_a011::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c001_a012")]
                                    "c001_a012" => Some(c001::c001_a012::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c001_a018")]
                                    "c001_a018" => Some(c001::c001_a018::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c001_a023")]
                                    "c001_a023" => Some(c001::c001_a023::solve_challenge as SolveChallengeFn),
                                    
                                    
                                    _ => Option::<SolveChallengeFn>::None,
                                } {
                                    Some(solve_challenge) => {
                                        let challenge =
                                            tig_challenges::c001::Challenge::generate_instance_from_vec(
                                                seeds,
                                                &job.settings.difficulty,
                                            )
                                            .unwrap();
                                        match solve_challenge(&challenge) {
                                            Ok(Some(solution)) => {
                                                challenge.verify_solution(&solution).is_err()
                                            }
                                            _ => true,
                                        }
                                    }
                                    None => false,
                                }
                            }
                            "c002" => {
                                type SolveChallengeFn =
                                    fn(
                                        &tig_challenges::c002::Challenge,
                                    )
                                        -> anyhow::Result<Option<tig_challenges::c002::Solution>>;
                                match match job.settings.algorithm_id.as_str() {
                                    #[cfg(feature = "c002_a001")]
                                    "c002_a001" => Some(c002::c002_a001::solve_challenge as SolveChallengeFn),
                                    
                                    
                                    _ => Option::<SolveChallengeFn>::None,
                                } {
                                    Some(solve_challenge) => {
                                        let challenge =
                                            tig_challenges::c002::Challenge::generate_instance_from_vec(
                                                seeds,
                                                &job.settings.difficulty,
                                            )
                                            .unwrap();
                                        match solve_challenge(&challenge) {
                                            Ok(Some(solution)) => {
                                                challenge.verify_solution(&solution).is_err()
                                            }
                                            _ => true,
                                        }
                                    }
                                    None => false,
                                }
                            }
                            "c003" => {
                                type SolveChallengeFn =
                                    fn(
                                        &tig_challenges::c003::Challenge,
                                    )
                                        -> anyhow::Result<Option<tig_challenges::c003::Solution>>;
                                match match job.settings.algorithm_id.as_str() {
                                    #[cfg(feature = "c003_a001")]
                                    "c003_a001" => Some(c003::c003_a001::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c003_a007")]
                                    "c003_a007" => Some(c003::c003_a007::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c003_a019")]
                                    "c003_a019" => Some(c003::c003_a019::solve_challenge as SolveChallengeFn),
                                    
                                    
                                    _ => Option::<SolveChallengeFn>::None,
                                } {
                                    Some(solve_challenge) => {
                                        let challenge =
                                            tig_challenges::c003::Challenge::generate_instance_from_vec(
                                                seeds,
                                                &job.settings.difficulty,
                                            )
                                            .unwrap();
                                        match solve_challenge(&challenge) {
                                            Ok(Some(solution)) => {
                                                challenge.verify_solution(&solution).is_err()
                                            }
                                            _ => true,
                                        }
                                    }
                                    None => false,
                                }
                            }
                            "c004" => {
                                type SolveChallengeFn =
                                    unsafe extern "C" fn(
                                        seeds: *const u64, difficulty: *const VSODifficulty
                                    )
                                        -> u32;
                                match match job.settings.algorithm_id.as_str() {
                                    // #[cfg(feature = "c004_a014")]
                                    // "c004_a014" => Some(c004::c004_a014::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c004_a026")]
                                    "c004_a026" => Some(tig_native::vector_search::solve_optimax_cpp_full as SolveChallengeFn),
                                    
                                    
                                    _ => Option::<SolveChallengeFn>::None,
                                } {
                                        Some(solve_challenge) => {
                                            let vec_diff =  &job.settings.difficulty;
                                            let vs_difficulty = VSODifficulty {
                                                num_queries: vec_diff.get(0).unwrap().clone() as u32,
                                                better_than_baseline: vec_diff.get(1).unwrap().clone() as u32,
                                            };
                                            let is_valid = unsafe { solve_challenge(seeds.as_ptr(), &vs_difficulty)};
                                                match is_valid {
                                                    0 => {
                                                        //println!("Valid solution for num_queries: {}, better_than_baseline: {}", vs_difficulty.num_queries, vs_difficulty.baseline);
                                                        false
                                                    }
                                                    1 => {
                                                        //println!("No solution found for num_queries: {}, better_than_baseline: {} --> nb indexes missmatch the number of queries", num_queries, baseline);
                                                        true
                                                    }
                                                    2 => {
                                                        //println!("Algorithm error for num_queries: {}, better_than_baseline: {} --> out of bound", num_queries, baseline);
                                                        true
                                                    },
                                                    3 => {
                                                        //println!("Mismatch in results between solve_challenge and solve_challenge_outbound for num_queries: {}, better_than_baseline: {}", num_queries, baseline);
                                                        true
                                                    },
                                                    _v => {
                                                        //println!("Unkown value returned : {}", v);
                                                        true
                                                    }
                                                    
                                                }
                                        },
                                        None => false,
                                    }
                                },
                            _ => panic!("Unknown challenge id: {}", job.settings.challenge_id),
                        };


                        if skip {
                            continue;
                        }

                        
                        if let Ok(Some(solution_data)) = native_compute_solution(
                            &job.settings,
                            nonce,
                            wasm.as_slice(),
                            job.wasm_vm_config.max_memory,
                            job.wasm_vm_config.max_fuel,
                        ) {
                            if native_verify_solution(&job.settings, nonce, &solution_data.solution)
                                .is_ok()
                            {
                                if solution_data.calc_solution_signature()
                                    <= job.solution_signature_threshold
                                {
                                    let mut solutions_data = job.solutions_data.lock().await;
                                    (*solutions_data).insert(nonce, solution_data);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}