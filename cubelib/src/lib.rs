extern crate core;

#[cfg(feature = "cube")]
pub mod cube;
#[cfg(feature = "cube")]
pub mod cubie;
#[cfg(feature = "cube")]
pub mod algs;
#[cfg(feature = "cube")]
pub mod alignment;
#[cfg(feature = "solver")]
pub mod co;
#[cfg(feature = "solver")]
pub mod df_search;
#[cfg(feature = "solver")]
pub mod lookup_table;
#[cfg(feature = "solver")]
pub mod moveset;
#[cfg(feature = "solver")]
pub mod stream;
#[cfg(feature = "solver")]
pub mod coords;
#[cfg(feature = "solver")]
pub mod steps;
#[cfg(feature = "solver")]
pub mod tables;
//Target specific implementations
#[cfg(feature = "cube")]
mod avx2_cubie;
#[cfg(feature = "cube")]
mod wasm32_cubie;
#[cfg(feature = "cube")]
pub mod solution;
#[cfg(feature = "cube")]
pub mod defs;
