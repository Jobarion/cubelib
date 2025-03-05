use std::sync::mpsc::{Receiver, SyncSender};
use std::thread::{JoinHandle, park};
use itertools::Itertools;
use crate::cube::Cube333;
use crate::solver::solution::Solution;
use crate::solver_new::step::{Step, StepWorker, ToWorker, Worker};

const BUFFER_SIZE: usize = 10;

pub struct Sequential {
    steps: Vec<Box<dyn ToWorker + Send + 'static>>,
}

impl Sequential {
    pub fn new(steps: Vec<Box<dyn ToWorker + Send + 'static>>) -> Self {
        Self {
            steps,
        }
    }
}

struct SequentialWorker {
    workers: Vec<Box<dyn Worker>>,
}

impl Worker for SequentialWorker {
    fn start(&mut self) {
        self.workers.iter_mut()
            .for_each(|w| w.start());
    }

    fn stop(&mut self) -> Option<JoinHandle<()>> {
        todo!()
    }
}

impl ToWorker for Sequential {
    fn to_worker_box(mut self: Box<Self>, cube_state: Cube333, mut rc: Receiver<Solution>, tx_last: SyncSender<Solution>) -> Box<dyn Worker>
    where
        Self: Sized + Send + 'static
    {
        if self.steps.is_empty() {
            panic!("No");
        }
        let mut pairs = vec![];
        let (mut tx, mut rc_next) = std::sync::mpsc::sync_channel(BUFFER_SIZE);
        for i in 0..(self.steps.len() - 1) {
            pairs.push((Some(rc), Some(tx)));
            rc = rc_next;
            (tx, rc_next) = std::sync::mpsc::sync_channel(BUFFER_SIZE);
        }
        pairs.push((Some(rc), Some(tx_last)));
        let pairs = pairs.into_iter()
            .map(|(a, b)|(a.unwrap(), b.unwrap()));

        let workers = self.steps.into_iter().zip(pairs)
            .map(|(tw, (rc, tx))|tw.to_worker_box(cube_state.clone(), rc, tx))
            .collect_vec();
        Box::new(SequentialWorker {
            workers,
        })
    }
}