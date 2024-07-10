use std::fmt::{Debug, Display};
use std::hash::Hash;
use crate::algs::Algorithm;

pub trait Puzzle<Turn: PuzzleMove, Transformation: PuzzleMove>:
    Copy + Clone + Default + TurnableMut<Turn> + TransformableMut<Transformation> + InvertibleMut
    where
        Turn: Transformable<Transformation>,
{

}

impl<Turn: PuzzleMove, Transformation: PuzzleMove, P: Copy + Clone + Default> Puzzle<Turn, Transformation> for P
    where P: TurnableMut<Turn> + TransformableMut<Transformation> + InvertibleMut,
          Turn: Transformable<Transformation>
{

}

pub trait PuzzleMove: Sized + Copy + Clone + Hash + Eq + PartialEq + Debug + Display + From<usize> + Into<usize> + Invertible + 'static {
    fn all() -> &'static [Self];
    fn is_same_type(&self, other: &Self) -> bool;
}

pub trait TransformableMut<Transformation: PuzzleMove> {
    fn transform(&mut self, transformation: Transformation);
}

pub trait Transformable<Transformation: PuzzleMove> {
    fn transform(&self, transformation: Transformation) -> Self;
}

pub trait TurnableMut<Turn: PuzzleMove> {
    fn turn(&mut self, turn: Turn);
}

pub trait InvertibleMut {
    fn invert(&mut self);
}

pub trait Invertible {
    fn invert(&self) -> Self;
}

pub trait ApplyAlgorithm<Turn: PuzzleMove> {
    fn apply_alg(&mut self, alg: &Algorithm<Turn>);
}

impl<Turn: PuzzleMove, C: TurnableMut<Turn> + InvertibleMut> ApplyAlgorithm<Turn> for C {
    fn apply_alg(&mut self, alg: &Algorithm<Turn>) {
        for m in &alg.normal_moves {
            self.turn(*m);
        }
        if !alg.inverse_moves.is_empty() {
            self.invert();
            for m in &alg.inverse_moves {
                self.turn(*m);
            }
            self.invert();
        }
    }
}