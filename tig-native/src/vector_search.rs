use libc::size_t;

// Déclaration des structures et fonctions externes
#[repr(C)]
pub struct VSODifficulty {
    pub num_queries: u32,
    pub better_than_baseline: u32,
}

#[repr(C)]
pub struct VSOSolution {
    pub indexes: *mut size_t,
    pub len: size_t,
}

#[repr(C)]
pub struct VSOChallenge {
    pub seeds: [u64; 8],
    pub difficulty: VSODifficulty,
    pub vector_database: *mut *mut f32, 
    pub query_vectors: *mut *mut f32,
    pub vector_database_size: size_t,
    pub query_vectors_size: size_t,
    pub max_distance: f32,
}

#[link(name = "stdc++")]
#[link(name = "cpp_cuda")]
extern "C" {
    pub fn solve_optimax_cpp(challenge: *const VSOChallenge, solution: *mut VSOSolution);
    pub fn generate_instance_vs(seeds: *const u64, difficulty: *const VSODifficulty) -> *mut VSOChallenge;
    pub fn solve_optimax_cpp_full(seeds: *const u64, difficulty: *const VSODifficulty) -> u32;
    pub fn free_vso_challenge(challenge: *const VSOChallenge);
}

