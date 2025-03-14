use std::{mem, thread};
use std::thread::JoinHandle;

use crate::cube::Cube333;
use crate::solver::solution::Solution;
use crate::solver_new::{Receiver, Sender, SendError};
use crate::solver_new::group::StepPredicate;

pub enum ThreadState<O> {
    None,
    PreStart(Box<dyn Run<O> + Send>),
    PostStart(JoinHandle<O>),
}

pub type SubmitFunction = Box<dyn FnMut(&Solution) -> Result<(), SendError<Solution>>>;

impl <O> Default for ThreadState<O> {
    fn default() -> Self {
        Self::None
    }
}

impl <O: Send + 'static> Worker<O> for ThreadState<O> {
    fn start(&mut self) {
        *self = match mem::take(self) {
            ThreadState::None => ThreadState::None,
            ThreadState::PreStart(mut runner) => ThreadState::PostStart(thread::spawn(move || runner.run())),
            ThreadState::PostStart(x) => ThreadState::PostStart(x),
        }
    }

    fn stop(&mut self) -> Option<JoinHandle<O>> {
        match mem::take(self) {
            ThreadState::None => None,
            ThreadState::PreStart(x) => {
                *self = ThreadState::PreStart(x);
                None
            }
            ThreadState::PostStart(jh) => Some(jh),
        }
    }
}

pub trait Run<T> {
    fn run(&mut self) -> T;
}

pub trait Worker<O: Send + 'static> {
    fn start(&mut self);
    fn stop(&mut self) -> Option<JoinHandle<O>>;
}

pub trait ToWorker: Send {
    fn to_worker(self: Self, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>, additional_predicates: Vec<Box<dyn StepPredicate>>) -> Box<dyn Worker<()> + Send> where Self: Send + 'static + Sized {
        Box::new(self).to_worker_box(cube_state, rc, tx, additional_predicates)
    }
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>, additional_predicates: Vec<Box<dyn StepPredicate>>) -> Box<dyn Worker<()> + Send> where Self: Send + 'static;
}
