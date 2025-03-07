use std::collections::HashSet;

use crate::algs::Algorithm;
use crate::cube::{Cube333, Turn333};
use crate::cube::turn::Direction;
use crate::solver::solution::Solution;
use crate::solver_new::{Receiver, Sender};
use crate::solver_new::thread_util::{Run, ThreadState, ToWorker, Worker};

pub struct FilterDup;
pub struct FilterLastMoveNotPrime;
pub struct Filter(pub Box<dyn Fn(&Solution, &Cube333) -> bool + Send>);

struct FilterDupWorker {
    rc: Receiver<Solution>,
    tx: Sender<Solution>,
    seen: HashSet<Algorithm>,
}

impl Run<()> for FilterDupWorker {
    fn run(&mut self) -> () {
        loop {
            match self.rc.recv() {
                Ok(sol) => {
                    if self.seen.insert(sol.clone().into()) {
                        if let Err(_) = self.tx.send(sol) {
                            return;
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }
    }
}

impl ToWorker for FilterDup {
    fn to_worker_box(self: Box<Self>, _: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Send + 'static
    {
        Box::new(ThreadState::PreStart(Box::new(FilterDupWorker {
            rc,
            tx,
            seen: Default::default(),
        })))
    }
}

pub trait StepFilter where Self: 'static {
    fn filter(&self, sol: &Solution, cube: &Cube333) -> bool;
    fn create_worker(self, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send> where Self: Send + Sized {
        Box::new(ThreadState::PreStart(Box::new(StepFilterWorker {
            rc,
            tx,
            filter: Box::new(self),
            cube: cube_state,
        })))
    }
}

struct StepFilterWorker {
    filter: Box<dyn StepFilter + Send>,
    rc: Receiver<Solution>,
    tx: Sender<Solution>,
    cube: Cube333
}

impl Run<()> for StepFilterWorker {
    fn run(&mut self) -> () {
        loop {
            match self.rc.recv() {
                Ok(sol) => {
                    if self.filter.filter(&sol, &self.cube) {
                        if let Err(_) = self.tx.send(sol) {
                            return;
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }
    }
}

struct FilterWorker {
    rc: Receiver<Solution>,
    tx: Sender<Solution>,
    predicate: Box<dyn Fn(&Solution, &Cube333) -> bool + Send>,
    cube: Cube333,
}

impl Run<()> for FilterWorker {
    fn run(&mut self) -> () {
        loop {
            match self.rc.recv() {
                Ok(sol) => {
                    if (self.predicate)(&sol, &self.cube) {
                        if let Err(_) = self.tx.send(sol) {
                            return;
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }
    }
}

impl ToWorker for Filter {
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Send + 'static
    {
        Box::new(ThreadState::PreStart(Box::new(FilterWorker {
            rc,
            tx,
            predicate: self.0,
            cube: cube_state,
        })))
    }
}

impl ToWorker for FilterLastMoveNotPrime {
    fn to_worker_box(self: Box<Self>, c: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Send + 'static
    {
        Filter(Box::new(check_solution)).to_worker(c, rc, tx)
    }
}

fn check_solution(sol: &Solution, _: &Cube333) -> bool {
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
    let alg: Algorithm = sol.clone().into();
    non_prime_end(&alg.normal_moves) && non_prime_end(&alg.inverse_moves)
}