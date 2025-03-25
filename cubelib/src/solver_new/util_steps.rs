use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::algs::Algorithm;
use crate::cube::{Cube333, Turn333};
use crate::cube::turn::Direction;
use crate::defs::StepKind;
use crate::solver::solution::Solution;
use crate::solver_new::group::{StepPredicate, StepPredicateResult};

pub struct FilterDup(RefCell<HashSet<Algorithm>>);
pub struct FilterLastMoveNotPrime;
pub struct Filter(pub Box<dyn Fn(&Solution, &Cube333) -> bool + Send>);
pub struct FilterFirstN(AtomicUsize);
pub struct FilterFirstNStepVariant {
    kind: StepKind,
    n: usize,
    found: RefCell<HashMap<Algorithm, usize>>,
}

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

impl FilterFirstNStepVariant {
    pub fn new(kind: StepKind, n: usize) -> Box<dyn StepPredicate> {
        Box::new(Self {
            kind,
            n,
            found: RefCell::new(Default::default()),
        })
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

impl StepPredicate for FilterFirstNStepVariant {
    fn check_solution(&self, solution: &Solution) -> StepPredicateResult {
        if self.kind == StepKind::FIN {
            return StepPredicateResult::Accepted;
        }
        let mut alg_up_to_kind = Algorithm::new();
        let mut found = false;
        for step in &solution.steps {
            alg_up_to_kind = alg_up_to_kind + step.alg.clone();
            if step.kind == self.kind {
                found = true;
                break;
            }
        }
        if !found {
            return StepPredicateResult::Accepted;
        }
        let normalized_alg = match self.kind {
            StepKind::FR | StepKind::FIN | StepKind::Other(_) => alg_up_to_kind,
            _ => to_last_move_non_prime_alg(alg_up_to_kind)
        };
        let mut found = self.found.borrow_mut();
        let entry = found.entry(normalized_alg);
        let entry = entry.or_insert(0);
        *entry += 1;
        if *entry <= self.n {
            StepPredicateResult::Accepted
        } else {
            StepPredicateResult::Rejected
        }
    }
}

fn to_last_move_non_prime_alg(mut alg: Algorithm) -> Algorithm {
    fn to_non_prime(vec: &mut Vec<Turn333>) {
        if vec.is_empty() {
            return;
        }
        let len = vec.len();
        let last = vec.last_mut().unwrap();
        if last.dir == Direction::CounterClockwise {
            last.dir = Direction::Clockwise;
        }
        if len > 1 {
            let last_face = last.face;
            let before_last = &mut vec[len - 2];
            if before_last.face == last_face.opposite() && before_last.dir == Direction::CounterClockwise{
                before_last.dir = Direction::Clockwise;
            }
        }
    }
    to_non_prime(&mut alg.normal_moves);
    to_non_prime(&mut alg.inverse_moves);
    alg
}