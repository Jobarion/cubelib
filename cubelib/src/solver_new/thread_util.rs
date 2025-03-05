use std::{mem, thread};
use std::thread::JoinHandle;

use crate::cube::Cube333;
use crate::solver::solution::Solution;
use crate::solver_new::step::{Receiver, Sender};

pub enum ThreadState<I: Run<O>, O> {
    None,
    PreStart(I),
    PostStart(JoinHandle<O>),
}

impl <I: Run<O>, O> Default for ThreadState<I, O> {
    fn default() -> Self {
        Self::None
    }
}

impl <I: Run<O> + Send + 'static, O: Send + 'static> Worker<O> for ThreadState<I, O> {
    fn start(&mut self) {
        *self = match mem::take(self) {
            ThreadState::None => ThreadState::None,
            ThreadState::PreStart(mut x) => ThreadState::PostStart(thread::spawn(move ||x.run())),
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

pub trait ToWorker {
    fn to_worker(self: Self, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send> where Self: Send + 'static + Sized {
        Box::new(self).to_worker_box(cube_state, rc, tx)
    }
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send> where Self: Send + 'static;
}
