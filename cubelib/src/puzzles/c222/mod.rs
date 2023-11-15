#[cfg(feature = "solver")]
pub mod coords;
#[cfg(feature = "solver")]
pub mod steps;

use crate::puzzles::cube::CubeCornersOdd;

pub type Transformation222 = crate::puzzles::cube::CubeTransformation;
pub type Turn222 = crate::puzzles::cube::CubeOuterTurn;
pub type Cube222 = CubeCornersEven;