extern crate libc;
use libc::{c_uint, c_int};

#[repr(C)]
#[derive(Clone)]
#[derive(Debug)]
pub struct Challenge {
    pub max_weight: c_uint,
    pub min_value: c_uint,
    pub num_items: c_uint,
    pub values: *const c_uint,
    pub weights: *const c_uint,
}

#[repr(C)]
pub struct SolutionC {
    pub items: *mut c_int,
}

#[link(name = "stdc++")]
#[link(name = "c")]
extern "C" {
    fn solve_knapmaxxing_v2_cuda(challenge: Challenge, solution: SolutionC);
}

pub fn knapsack_knapmaxxing_solver(challenge: &Challenge) -> Option<Vec<usize>> {
    let mut solution = vec![-1; challenge.num_items as usize];
    let c_solution = SolutionC {
        items: solution.as_mut_ptr(),
    };
    unsafe {
        solve_knapmaxxing_v2_cuda(challenge.clone(), c_solution);
    }
    
    // Si la première valeur est -2, retourner None
    if solution[0] == -2 {
        return None;
    }

    // Supprimer les valeurs -1 et convertir en Vec<usize>
    let items: Vec<usize> = solution
        .into_iter()
        .filter(|&value| value != -1)
        .map(|value| value as usize)
        .collect();
    
    Some(items)
}