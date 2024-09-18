pub mod dynamic;
pub use dynamic as c003_a001;
pub mod knapmaxxing;

pub use knapmaxxing as c003_a007;

pub mod knap_super_bb;
pub mod quick_knap;

pub mod knapheudp;
pub use knapheudp as c003_a019;

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use tig_challenges::{knapsack::*, *};

    // #[test]
    fn test_vehicle() {
        let difficulties = [[55, 15]];

        let mut counter = 0;
        let mut sol = 0;
        let mut invalid = 0;
        let mut time = Duration::from_secs(0);
        for diff_arr in difficulties {
            for nonce in 0..100000 {
                counter += 1;
                let difficulty = Difficulty {
                    num_items: diff_arr[0],
                    better_than_baseline: diff_arr[1] as u32,
                };
                let seeds = [nonce; 8];
                let challenge = Challenge::generate_instance(seeds, &difficulty).unwrap();

                let start_time = Instant::now();
                match quick_knap::solve_challenge(&challenge) {
                    Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                        Ok(_) => {
                            sol += 1;
                        }
                        Err(e) => {}
                    },
                    Ok(None) => invalid += 1,
                    Err(e) => invalid += 1,
                };
                time += start_time.elapsed();
            }
        }
        println!("#instances: {:?}, #solutions: {:?}, #invalid: {:?}, #errors: 0, total time: {:?}, average time: {:?}",counter,sol,invalid,time,(time/counter));
    }
}
