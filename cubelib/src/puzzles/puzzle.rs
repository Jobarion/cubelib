use crate::algs::Algorithm;
use crate::puzzles::c333::{Transformation333, Turn333};

pub trait TransformableMut {
    fn transform(&mut self, transformation: Transformation333);
}

pub trait Transformable {
    fn transform(&self, transformation: Transformation333) -> Self;
}

pub trait TurnableMut {
    fn turn(&mut self, turn: Turn333);
}

pub trait InvertibleMut {
    fn invert(&mut self);
}

pub trait Invertible {
    fn invert(&self) -> Self;
}

pub trait ApplyAlgorithm {
    fn apply_alg(&mut self, alg: &Algorithm);
}

impl<C: TurnableMut + InvertibleMut> ApplyAlgorithm for C {
    fn apply_alg(&mut self, alg: &Algorithm) {
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