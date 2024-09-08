mod optimax_native;
mod benchmarker_outbound;

pub use optimax_native::solve_challenge;
pub use optimax_native::solve_challenge_native;
pub use benchmarker_outbound::solve_challenge_outbound;
#[cfg(feature = "cuda")]
pub use optimax_native::{cuda_solve_challenge, KERNEL};