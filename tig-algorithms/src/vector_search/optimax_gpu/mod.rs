mod optimax_native;
pub use optimax_native::solve_challenge;
#[cfg(feature = "cuda")]
pub use optimax_native::{cuda_solve_challenge, KERNEL};