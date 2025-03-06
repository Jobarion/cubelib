use std::{mem, thread};
use std::cmp::{max, min};
use std::collections::HashSet;
use std::thread::JoinHandle;

use crossbeam::channel::{Select, TrySendError};
use itertools::Itertools;

use crate::cube::Cube333;
use crate::solver::solution::Solution;
use crate::solver_new::step::*;
use crate::solver_new::thread_util::*;

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
    workers: Vec<Box<dyn Worker<()> + Send>>,
}

impl Worker<()> for SequentialWorker {
    fn start(&mut self) {
        self.workers.iter_mut()
            .for_each(|w| w.start());
    }

    fn stop(&mut self) -> Option<JoinHandle<()>> {
        let workers = mem::take(&mut self.workers);
        Some(thread::spawn(move||{
            let mut to_join = vec![];
            for mut w in workers {
                if let Some(handle) = w.stop() {
                    to_join.push(handle);
                }
            }
            for handle in to_join {
                handle.join().unwrap();
            }
        }))
    }
}

impl ToWorker for Sequential {
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, mut rc: Receiver<Solution>, tx_last: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Sized + Send + 'static
    {
        assert!(!self.steps.is_empty());
        let mut pairs = vec![];
        let (mut tx, mut rc_next) = bounded_channel(BUFFER_SIZE);
        for _ in 0..(self.steps.len() - 1) {
            pairs.push((Some(rc), Some(tx)));
            rc = rc_next;
            (tx, rc_next) = bounded_channel(BUFFER_SIZE);
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

pub struct Parallel {
    steps: Vec<Box<dyn ToWorker + Send + 'static>>,
}

impl Parallel {
    pub fn new(steps: Vec<Box<dyn ToWorker + Send + 'static>>) -> Self {
        Self {
            steps,
        }
    }
}

struct Broadcaster {
    sinks: Vec<Sink>,
    source: Receiver<Solution>,
    buffer: Vec<Solution>
}

struct Sink {
    sink: Sender<Solution>,
    buffer_position: usize,
}

impl Broadcaster {
    pub fn new(source: Receiver<Solution>, sinks: Vec<Sender<Solution>>) -> Self {
        Self {
            sinks: sinks.into_iter()
                .map(|x|Sink {
                    sink: x,
                    buffer_position: 0,
                })
                .collect(),
            source,
            buffer: vec![],
        }
    }
}

impl Run<()> for Broadcaster {
    fn run(&mut self) {
        while !self.sinks.is_empty()  {
            let mut lowest = self.buffer.len();
            let mut highest = 0;
            let mut to_remove = vec![];
            for (sink, id) in self.sinks.iter_mut().zip(0..) {
                lowest = min(lowest, sink.buffer_position);
                highest = max(highest, sink.buffer_position);
                while sink.buffer_position < self.buffer.len() {
                    match sink.sink.try_send(self.buffer[sink.buffer_position].clone()) {
                        Ok(_) => {
                            sink.buffer_position += 1;
                        }
                        Err(TrySendError::Disconnected(_)) => {
                            to_remove.push(id);
                            break;
                        }
                        _ => {
                            break;
                        }
                    }
                }
            }
            for id in to_remove.into_iter().rev() {
                self.sinks.remove(id);
            }
            if lowest > 0 {
                self.buffer.drain(0..lowest);
                for sink in self.sinks.iter_mut() {
                    sink.buffer_position -= lowest;
                }
                highest -= lowest;
            }
            if self.buffer.is_empty() {
                // The buffer is empty, we need a new element. If there is none, we can abort immediately
                if let Ok(x) = self.source.recv() {
                    self.buffer.push(x)
                } else {
                    self.sinks.clear();
                    return;
                }
            } else if highest == self.buffer.len() {
                // Someone reached the end of the buffer, so we'll try to fetch a new element. We don't block to serve the workers that did not reach the end of the buffer yet.
                // TODO this can spin, use select to do better
                match self.source.try_recv() {
                    Ok(s) => self.buffer.push(s),
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => {
                        self.sinks.retain(|s|s.buffer_position < self.buffer.len());
                        break;
                    },
                }
            }
        }
    }
}

pub struct FifoSampler {
    sources: Vec<Receiver<Solution>>,
    sink: Sender<Solution>,
}

impl FifoSampler {
    pub fn new(sink: Sender<Solution>, sources: Vec<Receiver<Solution>>) -> Self {
        Self {
            sources,
            sink,
        }
    }
}

impl Run<()> for FifoSampler {
    fn run(&mut self) -> () {
        let mut sel = Select::new();

        for source in &self.sources {
            sel.recv(source);
        }

        let mut connected = self.sources.len();
        while connected > 0 {
            let index = sel.ready();

            match self.sources[index].try_recv() {
                Ok(res) => {
                    if let Err(_) = self.sink.send(res) {
                        return;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    sel.remove(index);
                    connected -= 1;
                }
                _ => {}
            }
        }
    }
}

pub struct InOrderSampler {
    sources: Vec<Receiver<Solution>>,
    sink: Sender<Solution>,
}

impl InOrderSampler {
    pub fn new(sink: Sender<Solution>, sources: Vec<Receiver<Solution>>) -> Self {
        Self {
            sources,
            sink,
        }
    }
}

impl Run<()> for InOrderSampler {
    fn run(&mut self) -> () {
        let mut cache: Vec<Option<Solution>> = vec![None; self.sources.len()];
        let mut sel = Select::new();
        let mut target_length = 0;
        let mut active = self.sources.len();
        for s in &self.sources {
            sel.recv(s);
        }
        let mut dead = HashSet::new();
        while dead.len() < self.sources.len() {
            // trace!("{active} {target_length} {cache:?}");
            if active == 0 {
                target_length = cache.iter().filter_map(|x| x.as_ref().map(|x| x.len())).min().unwrap_or(target_length + 1);
                for idx in 0..cache.len() {
                    if cache[idx].as_ref().filter(|x|x.len() <= target_length).is_some() {
                        if let Err(_) = self.sink.send(cache[idx].take().unwrap()) {
                            return;
                        }
                    }
                }
                active = self.sources.len();
                sel = Select::new();
                for s in &self.sources {
                    sel.recv(s);
                }
            }
            let index = sel.ready();
            if cache[index].is_some() {
                sel.remove(index);
                active -= 1;
                continue;
            }
            match self.sources[index].try_recv() {
                Ok(res) => {
                    if res.len() > target_length {
                        sel.remove(index);
                        cache[index] = Some(res);
                        active -= 1;
                        continue
                    }
                    if let Err(_) = self.sink.send(res) {
                        return;
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    sel.remove(index);
                    dead.insert(index);
                    active -= 1;
                }
                _ => {}
            }
        }
    }
}

struct ParallelWorker {
    workers: Vec<Box<dyn Worker<()> + Send>>,
    broadcaster: ThreadState<()>,
    sampler: ThreadState<()>,
}

impl Worker<()> for ParallelWorker {
    fn start(&mut self) {
        self.broadcaster.start();
        self.workers.iter_mut()
            .for_each(|w| w.start());
        self.sampler.start();
    }

    fn stop(&mut self) -> Option<JoinHandle<()>> {
        let workers = mem::take(&mut self.workers);
        let mut broadcaster = mem::take(&mut self.broadcaster);
        let mut sampler = mem::take(&mut self.sampler);

        Some(thread::spawn(move||{
            let mut to_join = vec![];
            if let Some(handle) = broadcaster.stop() {
                to_join.push(handle);
            }
            if let Some(handle) = sampler.stop() {
                to_join.push(handle);
            }
            for mut w in workers {
                if let Some(handle) = w.stop() {
                    to_join.push(handle);
                }
            }
            for handle in to_join {
                handle.join().unwrap();
            }
        }))
    }
}

impl ToWorker for Parallel {
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Send + 'static
    {
        let mut workers = vec![];
        let mut inputs = vec![];
        let mut outputs = vec![];
        for step in self.steps.into_iter() {
            let (tx0, rc0) = bounded_channel(BUFFER_SIZE);
            let (tx1, rc1) = bounded_channel(BUFFER_SIZE);
            workers.push(step.to_worker_box(cube_state.clone(), rc0, tx1));
            inputs.push(rc1);
            outputs.push(tx0);
        }
        Box::new(ParallelWorker {
            broadcaster: ThreadState::PreStart(Box::new(Broadcaster::new(rc, outputs))),
            workers,
            sampler: ThreadState::PreStart(Box::new(InOrderSampler::new(tx, inputs))),
        })
    }
}