pub mod clarke_wright;
pub mod clarke_wright_super;
pub mod cw_heuristic;
pub mod cw_super_too;
pub mod cw_ultimate;
pub mod super_darwin;

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::{
        sync::{Arc, Mutex},
        time::Instant,
    };
    use tig_challenges::{vehicle_routing::*, *};

    #[test]
    fn test_once() {
        let difficulty = Difficulty {
            num_nodes: 60,
            better_than_baseline: 270,
        };
        let seeds = [0; 8];
        let challenge = Challenge::generate_instance(seeds, &difficulty).unwrap();

        let time1 = Instant::now();
        let result1 = match clarke_wright::solve_challenge(&challenge) {
            Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                Ok(_) => String::from("Valid solution"),
                Err(e) => format!("Invalid solution: {}", e),
            },
            Ok(None) => String::from("No solution"),
            Err(e) => format!("Algorithm error: {}", e),
        };
        let elapsed1 = time1.elapsed();

        let time2 = Instant::now();
        let result2 = match super_darwin::solve_challenge(&challenge) {
            Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                Ok(_) => String::from("Valid solution"),
                Err(e) => format!("Invalid solution: {}", e),
            },
            Ok(None) => String::from("No solution"),
            Err(e) => format!("Algorithm error: {}", e),
        };
        let elapsed2 = time2.elapsed();

        println!(
            "{:?}: {:?} / {:?}: {:?}",
            elapsed1.as_millis(),
            result1,
            elapsed2.as_millis(),
            result2
        );
    }

    // #[test]
    fn test_multi_nonces() {
        let difficulties = [[60, 270]];
        let counter = Arc::new(Mutex::new(0));

        for diff_arr in difficulties {
            (0..100000).into_par_iter().for_each(|nonce| {
                let difficulty = Difficulty {
                    num_nodes: diff_arr[0],
                    better_than_baseline: diff_arr[1] as u32,
                };
                let seeds = [nonce; 8];
                let challenge = Challenge::generate_instance(seeds, &difficulty).unwrap();

                match super_darwin::solve_challenge(&challenge) {
                    Ok(Some(solution)) => match challenge.verify_solution(&solution) {
                        Ok(_) => {
                            let mut counter = counter.lock().unwrap();
                            *counter += 1;
                            println!("Total solutions: {:?}", *counter);
                        }
                        Err(_e) => {}
                    },
                    Ok(None) => {}
                    Err(_e) => {}
                };
            });
        }
        println!("Total solutions: {:?}", *counter.lock().unwrap());
    }
}
