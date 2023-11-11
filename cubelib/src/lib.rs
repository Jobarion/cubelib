extern crate core;

pub mod algs;
mod alignment;
pub mod defs;
#[cfg(feature = "333")]
pub mod co;
#[cfg(feature = "solver")]
pub mod steps;
#[cfg(feature = "solver")]
pub mod solver;
#[cfg(target_arch = "wasm32")]
mod wasm_util;
pub mod puzzles;
