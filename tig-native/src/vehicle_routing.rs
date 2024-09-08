use std::os::raw::{c_int, c_uint};


#[repr(C)]
pub struct CWChallenge {
    pub seed: u64,
    pub demands: *const c_int,
    pub distance_matrix: *const c_int,
    pub max_total_distance: c_int,
    pub max_capacity: c_int,
    pub num_nodes: c_uint,
}

#[repr(C)]
pub struct CWSolution {
    pub routes: *mut *mut c_int,
    pub route_lengths: *mut c_int,
    pub num_routes: c_int,
}
