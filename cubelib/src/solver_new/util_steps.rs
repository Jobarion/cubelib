use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::algs::Algorithm;
use crate::cube::{Cube333, Turn333};
use crate::cube::turn::Direction;
use crate::solver::solution::Solution;
use crate::solver_new::group::{StepPredicate, StepPredicateResult};

pub struct FilterDup(RefCell<HashSet<Algorithm>>);
pub struct FilterLastMoveNotPrime;
pub struct Filter(pub Box<dyn Fn(&Solution, &Cube333) -> bool + Send>);
pub struct FilterFirstN(AtomicUsize);

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

impl FilterFirstN {
    pub fn new(n: usize) -> Box<dyn StepPredicate> {
        Box::new(Self(AtomicUsize::new(n)))
    }
}

impl StepPredicate for FilterDup {
    fn check_solution(&self, solution: &Solution) -> StepPredicateResult {
        if self.0.borrow_mut().insert(solution.clone().into()) {
            StepPredicateResult::Accepted
        } else {
            StepPredicateResult::Rejected
        }
    }
}

impl StepPredicate for FilterLastMoveNotPrime {
    fn check_solution(&self, solution: &Solution) -> StepPredicateResult {
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
        if non_prime_end(&alg.normal_moves) && non_prime_end(&alg.inverse_moves) {
            StepPredicateResult::Accepted
        } else {
            StepPredicateResult::Rejected
        }
    }
}

impl StepPredicate for FilterFirstN {
    fn check_solution(&self, _: &Solution) -> StepPredicateResult {
        let mut v = self.0.load(Ordering::Relaxed);
        if v == 0 {
            return StepPredicateResult::Closed;
        }
        loop {
            match self.0.compare_exchange(v, v - 1, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => {
                    return StepPredicateResult::Accepted;
                }
                Err(old) => {
                    if old == 0 {
                        return StepPredicateResult::Closed;
                    }
                    v = old;
                }
            }
        }
    }
}
