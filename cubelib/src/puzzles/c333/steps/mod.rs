#[cfg(feature = "333eo")]
pub mod eo;
#[cfg(feature = "333dr")]
pub mod dr;
#[cfg(feature = "333htr")]
pub mod htr;
#[cfg(feature = "333fr")]
pub mod fr;
#[cfg(feature = "333finish")]
pub mod finish;
pub mod solver;
#[cfg(any(feature = "333eo", feature = "333dr", feature = "333htr", feature = "333fr", feature = "333finish"))]
pub mod tables;

pub type Step333<'a> = crate::steps::step::Step<'a, crate::puzzles::c333::Turn333, crate::puzzles::c333::Transformation333, crate::puzzles::c333::Cube333, crate::solver::moveset::TransitionTable333>;
pub type MoveSet333 = crate::solver::moveset::MoveSet<crate::puzzles::c333::Turn333, crate::solver::moveset::TransitionTable333>;
//Trait aliases pls