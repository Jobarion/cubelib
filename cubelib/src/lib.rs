#![feature(lazy_type_alias)]
#![feature(lock_value_accessors)]
#![allow(incomplete_features)]
extern crate core;

#[macro_use]
#[cfg(feature = "solver")]
extern crate num_derive;

pub mod algs;
pub mod cube;
pub mod defs;
mod simd_util;
#[cfg(feature = "solver")]
pub mod solver;
#[cfg(feature = "multi-path-channel-solver")]
pub mod solver_new;
#[cfg(feature = "solver")]
pub mod steps;
#[cfg(target_arch = "wasm32")]
mod wasm_util;
