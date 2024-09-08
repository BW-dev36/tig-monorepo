use super::{Job, NonceIterator};
use crate::utils;
use std::sync::Arc;
use tig_algorithms::{c001, c002, c003, c004};
use tig_challenges::ChallengeTrait;
use tig_worker::{compute_solution, verify_solution};
use tokio::{spawn, sync::Mutex, task::yield_now};
use utils::time;


pub async fn execute(nonce_iterators: Vec<Arc<Mutex<NonceIterator>>>, job: &Job, wasm: &Vec<u8>) {
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
                                    fn(
                                        &tig_challenges::c004::Challenge,
                                    )
                                        -> anyhow::Result<Option<tig_challenges::c004::Solution>>;
                                match match job.settings.algorithm_id.as_str() {
                                    #[cfg(feature = "c004_a014")]
                                    "c004_a014" => Some(c004::c004_a014::solve_challenge as SolveChallengeFn),
                                    #[cfg(feature = "c004_a026")]
                                    "c004_a026" => Some(c004::c004_a026::solve_challenge as SolveChallengeFn),
                                    
                                    
                                    _ => Option::<SolveChallengeFn>::None,
                                } {
                                    Some(solve_challenge) => {
                                        let challenge =
                                            tig_challenges::c004::Challenge::generate_instance_from_vec(
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
                            _ => panic!("Unknown challenge id: {}", job.settings.challenge_id),
                        };
                        if skip {
                            continue;
                        }




                        
                        if let Ok(Some(solution_data)) = compute_solution(
                            &job.settings,
                            nonce,
                            wasm.as_slice(),
                            job.wasm_vm_config.max_memory,
                            job.wasm_vm_config.max_fuel,
                        ) {
                            if verify_solution(&job.settings, nonce, &solution_data.solution)
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