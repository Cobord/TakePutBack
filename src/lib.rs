pub mod general;

#[cfg(feature = "petgraph")]
pub mod graphs;

pub mod vectors;

pub use general::TakePutBack;
