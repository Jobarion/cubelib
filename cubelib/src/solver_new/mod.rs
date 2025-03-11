use crate::cube::{Cube333, Transformation333};
use crate::defs::StepKind;
use crate::solver::solution::Solution;
use crate::solver_new::group::StepGroup;
use crate::solver_new::step::{DFSParameters, MoveSet};
use crate::solver_new::thread_util::{ToWorker, Worker};
use crate::solver_new::util_steps::FilterDup;
use crate::steps::step::{PostStepCheck, PreStepCheck};

pub mod step;
pub mod eo;
pub mod dr;
pub mod group;
pub mod thread_util;
pub mod util_steps;
pub mod htr;
pub mod util_cube;

pub type Sender<T> = crossbeam::channel::Sender<T>;
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
pub type SendError<T> = crossbeam::channel::SendError<T>;
pub type RecvError = crossbeam::channel::RecvError;
pub type TryRecvError = crossbeam::channel::TryRecvError;

pub fn bounded_channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::bounded(size)
}

pub trait Step: PreStepCheck + PostStepCheck {
    fn get_dfs_parameters(&self) -> DFSParameters;
    fn get_moveset(&self, state: &Cube333, depth_left: usize) -> &'_ MoveSet;
    fn heuristic(&self, state: &Cube333, can_niss_switch: bool, depth_left: usize) -> usize;
    fn pre_step_trans(&self) -> &'_ Vec<Transformation333>;
    fn get_name(&self) -> (StepKind, String);
}

pub fn create_worker(cube: Cube333, step: Box<dyn ToWorker + Send + 'static>) -> (Box<dyn Worker<()> + Send + 'static>, Receiver<Solution>) {
    let (tx0, rc0) = bounded_channel(1);
    let (tx1, rc1) = bounded_channel(1);

    tx0.send(Solution::new()).unwrap();
    drop(tx0);

    (StepGroup::single_with_predicates(step, vec![FilterDup::new()])
        .to_worker_box(cube, rc0, tx1, vec![]), rc1)
}