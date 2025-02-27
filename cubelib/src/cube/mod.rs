use crate::cube::turn::{CubeOuterTurn, CubeTransformation};

mod cube;
mod cube_corners;
mod cube_edges;
pub mod turn;

pub type Cube333 = cube::Cube333;
pub type Turn333 = CubeOuterTurn;
pub type Transformation333 = CubeTransformation;
pub(crate) type CornerCube333 = cube_corners::CubeCornersOdd;
pub type Edge = turn::Edge;
pub type Corner = turn::Corner;
pub(crate) type EdgeCube333 = cube_edges::CenterEdgeCube;
pub type CubeFace = turn::CubeFace;
pub type CubeColor = turn::CubeColor;
pub type Direction = turn::Direction;
pub type CubeAxis = turn::CubeAxis;
// pub trait ApplyAlgorithm = turn::ApplyAlgorithm;
// pub trait Transformable = turn::Transformable;
// pub trait TransformableMut = turn::TransformableMut;
// pub trait Invertible = turn::Invertible;
// pub trait InvertibleMut = turn::InvertibleMut;
