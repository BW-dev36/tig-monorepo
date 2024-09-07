mod innovator_outbound;
pub use innovator_outbound::solve_challenge;
#[cfg(feature = "cuda")]
pub use innovator_outbound::{cuda_solve_challenge, KERNEL};
