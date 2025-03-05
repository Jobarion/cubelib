extern crate core;

#[macro_use]
#[cfg(feature = "solver")]
extern crate num_derive;

pub mod algs;
mod simd_util;
pub mod defs;
#[cfg(feature = "solver")]
pub mod steps;
#[cfg(feature = "solver")]
pub mod solver;
#[cfg(target_arch = "wasm32")]
mod wasm_util;
pub mod cube;
pub mod solver_new;
