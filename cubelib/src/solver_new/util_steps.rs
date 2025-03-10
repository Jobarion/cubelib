use std::cell::RefCell;
use std::collections::HashSet;

use crate::algs::Algorithm;
use crate::cube::{Cube333, Turn333};
use crate::cube::turn::Direction;
use crate::solver::solution::Solution;
use crate::solver_new::group::StepPredicate;

pub struct FilterDup(RefCell<HashSet<Algorithm>>);
pub struct FilterLastMoveNotPrime;
pub struct Filter(pub Box<dyn Fn(&Solution, &Cube333) -> bool + Send>);

impl FilterDup {
    pub fn new() -> Box<dyn StepPredicate> {
        Box::new(Self(RefCell::new(Default::default())))
    }
}

impl FilterLastMoveNotPrime {
    pub fn new() -> Box<dyn StepPredicate> {
        Box::new(Self)
    }
}

impl StepPredicate for FilterDup {
    fn check_solution(&self, solution: &Solution) -> bool {
        self.0.borrow_mut().insert(solution.clone().into())
    }
}

impl StepPredicate for FilterLastMoveNotPrime {
    fn check_solution(&self, solution: &Solution) -> bool {
        fn non_prime_end(vec: &Vec<Turn333>) -> bool {
            if vec.is_empty() {
                return true;
            }
            let last = vec.last().unwrap();
            if last.dir == Direction::CounterClockwise {
                return false;
            }
            if vec.len() > 1 {
                if vec[vec.len() - 2].face == last.face.opposite() {
                    return vec[vec.len() - 2].dir != Direction::CounterClockwise;
                }
            }
            return true;
        }
        let alg: Algorithm = solution.clone().into();
        non_prime_end(&alg.normal_moves) && non_prime_end(&alg.inverse_moves)
    }
}
