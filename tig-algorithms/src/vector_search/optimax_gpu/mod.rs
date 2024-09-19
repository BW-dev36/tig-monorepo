mod benchmarker_outbound;
pub use benchmarker_outbound::solve_challenge_old;
mod inbound;
pub use inbound::solve_challenge;

#[cfg(feature = "cuda")]
pub use benchmarker_outbound::{cuda_solve_challenge_old, KERNEL_OLD};
#[cfg(feature = "cuda")]
pub use inbound::{cuda_solve_challenge, KERNEL};