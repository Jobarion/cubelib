#[cfg(feature = "solver")]
pub mod steps;
mod cube;

pub type Transformation333 = crate::puzzles::cube::CubeTransformation;
pub type Turn333 = crate::puzzles::cube::CubeOuterTurn;
pub type Cube333 = cube::Cube333;
pub type CornerCube333 = crate::puzzles::cube::CornerCube;
pub type EdgeCube333 = crate::puzzles::cube::CenterEdgeCube;