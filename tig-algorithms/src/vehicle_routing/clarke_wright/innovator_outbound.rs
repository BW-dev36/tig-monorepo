/*!
Copyright 2024 Uncharted Trading Limited

Licensed under the TIG Innovator Outbound Game License v1.0 (the "License"); you 
may not use this file except in compliance with the License. You may obtain a copy 
of the License at

https://github.com/tig-foundation/tig-monorepo/tree/main/docs/licenses

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See the License for the specific
language governing permissions and limitations under the License.
*/
use tig_native::*;
use tig_challenges::vehicle_routing::*;
use std::ffi::c_void;
use std::os::raw::{c_int, c_uint};
use std::ptr;

#[link(name = "cpp_cuda")]
extern "C" {
    fn solve_clarke_wright_v1_cpp(challenge: *const CWChallenge, solution: *mut CWSolution);
}

pub fn solve_challenge(challenge: &Challenge) -> anyhow::Result<Option<Solution>> {
    let n = challenge.difficulty.num_nodes as usize;
    
    // Préparer les données pour CWChallenge
    let demands: Vec<c_int> = challenge.demands.iter().map(|&x| x as c_int).collect();
    let distance_matrix: Vec<c_int> = challenge.distance_matrix.iter().flatten().map(|&x| x as c_int).collect();
    
    let sat_challenge = CWChallenge {
        seed: challenge.seeds[0],
        demands: demands.as_ptr(),
        distance_matrix: distance_matrix.as_ptr(),
        max_total_distance: challenge.max_total_distance,
        max_capacity: challenge.max_capacity,
        num_nodes: n as c_uint,
    };
    
    let mut sat_solution = CWSolution {
        routes: ptr::null_mut(),
        route_lengths: ptr::null_mut(),
        num_routes: 0,
    };
    
    unsafe {
        solve_clarke_wright_v1_cpp(&sat_challenge as *const CWChallenge, &mut sat_solution as *mut CWSolution);
    }
    
    // Convertir CWSolution en Solution Rust
    let mut routes = Vec::new();
    unsafe {
        for i in 0..sat_solution.num_routes as usize {
            let route_length = *sat_solution.route_lengths.add(i) as usize;
            let route = std::slice::from_raw_parts(*sat_solution.routes.add(i), route_length);
            routes.push(route.iter().map(|&x| x as usize).collect::<Vec<usize>>());
        }
        
        // Libérer la mémoire allouée par C++
        for i in 0..sat_solution.num_routes as usize {
            libc::free(*sat_solution.routes.add(i) as *mut c_void);
        }
        libc::free(sat_solution.routes as *mut c_void);
        libc::free(sat_solution.route_lengths as *mut c_void);
    }
    
    Ok(Some(Solution { routes }))
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

