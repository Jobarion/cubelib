use std::cmp::min;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::thread::JoinHandle;

use log::trace;
use typed_builder::TypedBuilder;

use crate::algs::Algorithm;
use crate::cube::{Cube333, Transformation333, Turn333};
use crate::cube::turn::*;
use crate::defs::{NissSwitchType, StepKind};
use crate::solver::df_search::CancelToken;
use crate::solver::solution::{ApplySolution, Solution, SolutionStep};
use crate::solver_new::thread_util::*;
use crate::steps::step::{PostStepCheck, PreStepCheck};

#[derive(Clone, TypedBuilder)]
pub struct StepOptions<T: Clone, const DM: usize, const DAM: usize> {
    #[builder(default=0)]
    pub min_length: usize,
    #[builder(default=DM)]
    pub max_length: usize,
    #[builder(default, setter(strip_option))]
    pub min_absolute_length: Option<usize>,
    #[builder(default=Some(DAM).filter(|x|*x > 0), setter(strip_option))]
    pub max_absolute_length: Option<usize>,
    pub options: T,
}

// pub type Sender<T> = sync::mpsc::SyncSender<T>;
// pub type Receiver<T> = sync::mpsc::Receiver<T>;
// pub type SendError<T> = sync::mpsc::SendError<T>;
// pub type RecvError = sync::mpsc::RecvError;
// pub type TryRecvError = sync::mpsc::TryRecvError;

pub type Sender<T> = crossbeam::channel::Sender<T>;
pub type Receiver<T> = crossbeam::channel::Receiver<T>;
pub type SendError<T> = crossbeam::channel::SendError<T>;
pub type RecvError = crossbeam::channel::RecvError;
pub type TryRecvError = crossbeam::channel::TryRecvError;

pub fn bounded_channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    crossbeam::channel::bounded(size)
    // sync::mpsc::sync_channel(size)
}

impl <T: Clone, const DM: usize, const DAM: usize> Deref for StepOptions<T, DM, DAM> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.options
    }
}

impl <T: Clone, const DM: usize, const DAM: usize> Into<DFSParameters> for &StepOptions<T, DM, DAM> where for<'a> &'a T: Into<NissSwitchType> {
    fn into(self) -> DFSParameters {
        DFSParameters {
            niss_type: (&self.options).into(),
            min_moves: self.min_length,
            max_moves: self.max_length,
            absolute_min_moves: self.min_absolute_length,
            absolute_max_moves: self.max_absolute_length,
        }
    }
}

pub struct StepWorker {
    join_handle: Option<JoinHandle<()>>,
    cancel_token: Arc<CancelToken>,
    step_runner: ThreadState<StepIORunner, ()>,
}

impl Worker<()> for StepWorker {
    fn start(&mut self) {
        self.step_runner.start();
    }

    fn stop(&mut self) -> Option<JoinHandle<()>> {
        self.cancel_token.cancel();
        self.join_handle.take()
    }
}

struct StepIORunner {
    rc: Receiver<Solution>,
    tx: Sender<Solution>,
    input: Vec<Solution>,
    dfs_parameters: DFSParameters,
    current_length: usize,
    current_position: usize,
    step: Box<dyn Step + Send>,
    cancel_token: Arc<CancelToken>,
    cube_state: Cube333,
}

impl Run<()> for StepIORunner {
    fn run(&mut self) {
        let next = if let Ok(next) = self.rc.recv() {
            next
        } else {
            return;
        };
        self.input.push(next);

        while !self.cancel_token.is_cancelled() && self.current_length <= self.dfs_parameters.max_moves {
            trace!("[{}] Loop start {} {}", self.step.get_name().0, self.current_length, self.input.len());
            // If we need to fetch all inputs up to length X, we do this now
            match self.process_fetched() {
                Ok(Some(full_fetch_required_length)) => {
                    trace!("[{}] Requested fetch up to {full_fetch_required_length}", self.step.get_name().0);
                    while !self.cancel_token.is_cancelled() {
                        match self.rc.recv() {
                            Ok(next) => {
                                let len = next.len();
                                self.input.push(next);
                                if len > full_fetch_required_length {
                                    break
                                }
                            }
                            Err(_) => {
                                break
                            }
                        }
                    }
                },
                Ok(None) => {}
                Err(_) => {
                    return;
                }
            };
            if let Err(_) = self.process_fetched() {
                return;
            }
            self.current_position = 0;
            self.current_length += 1;
            if self.input[0].len() + self.current_length > self.dfs_parameters.absolute_max_moves.unwrap_or(usize::MAX) {
                break;
            }
        }
    }
}

impl StepIORunner {

    fn process_fetched(&mut self) -> Result<Option<usize>, SendError<Solution>> {
        let min_length = self.input[0].len();
        while !self.cancel_token.is_cancelled() && self.current_position < self.input.len() {
            let len = self.input[self.current_position].len();
            // We know that they are ordered by length, so we can abort immediately
            //trace!("[{}] \tCurrent state has length {len} (min is {min_length})", self.step.get_name().0);
            if len > self.current_length + min_length {
                return Ok(None);
            }
            self.find_solutions(self.cube_state.clone(), &self.input[self.current_position], self.current_length, self.dfs_parameters.niss_type)?;
            self.current_position += 1;
        }
        return Ok(Some(min_length + self.current_length));
    }

    fn submit_solution(&self, input: &Solution, result: Algorithm) -> Result<(), SendError<Solution>>{
        let mut input = input.clone();
        let (kind, variant) = self.step.get_name();

        input.add_step(SolutionStep {
            kind,
            variant,
            alg: result,
            comment: "".to_string(),
        });
        // Add alg
        self.tx.send(input)
    }

    // Finds solutions that exactly match the depth parameter. Does _not_ look for shorter ones
    pub fn find_solutions(&self, mut cube: Cube333, input: &Solution, depth: usize, niss_type: NissSwitchType) -> Result<(), SendError<Solution>> {
        cube.apply_solution(input);
        if !self.step.is_cube_ready(&cube) {
            return Ok(());
        }

        let alg: Algorithm = input.clone().into();
        let mut previous_normal = alg.normal_moves.last().cloned();
        let mut previous_inverse = alg.inverse_moves.last().cloned();
        let start_on_normal = input.ends_on_normal;

        for t in self.step.pre_step_trans().iter().cloned() {
            cube.transform(t);
            previous_normal = previous_normal.map(|m|m.transform(t));
            previous_inverse = previous_inverse.map(|m|m.transform(t));
        }

        let heuristic = self.step.heuristic(&cube, niss_type != NissSwitchType::Never);
        //trace!("[{}{}] \t{alg} is solvable in {}, depth is {depth}", self.step.get_name().0, self.step.get_name().1, heuristic);
        if self.step.heuristic(&cube, niss_type != NissSwitchType::Never) == 0 {
            //Only return a solution if we are allowed to return zero length solutions
            if depth == 0 {
                self.submit_solution(input, Algorithm::new())?;
            }
            return Ok(());
        }

        let cancel_token = self.cancel_token.as_ref();
        let iter: Box<dyn Iterator<Item = Algorithm>> = match niss_type {
            NissSwitchType::Never if start_on_normal => {
                Box::new(self.find_solutions_dfs(cube, depth, false, previous_normal, previous_inverse, cancel_token))
            },
            NissSwitchType::Never => {
                cube.invert();
                Box::new(self.find_solutions_dfs(cube, depth, false, previous_inverse, previous_normal, cancel_token)
                    .map(|alg| {
                        Algorithm {
                            normal_moves: alg.inverse_moves,
                            inverse_moves: alg.normal_moves
                        }
                    }))
            },
            NissSwitchType::Before => {
                let normal = self.find_solutions_dfs(cube.clone(), depth, false, previous_normal, previous_inverse, cancel_token);
                cube.invert();
                let inverse = self.find_solutions_dfs(cube, depth, false, previous_inverse, previous_normal, cancel_token)
                    .map(|alg| {
                        Algorithm {
                            normal_moves: alg.inverse_moves,
                            inverse_moves: alg.normal_moves
                        }
                    });
                Box::new(normal.chain(inverse))
            }
            NissSwitchType::Always => {
                let normal = self.find_solutions_dfs(cube.clone(), depth, true, previous_normal, previous_inverse, cancel_token);
                cube.invert();
                let inverse = self.find_solutions_dfs(cube, depth, false, previous_inverse, previous_normal, cancel_token)
                    .map(|alg| {
                        Algorithm {
                            normal_moves: alg.inverse_moves,
                            inverse_moves: alg.normal_moves
                        }
                    });
                Box::new(normal.chain(inverse))
            }
        };

        for alg in iter {
            let alg = alg.reverse();
            self.submit_solution(input, alg)?;
        }
        Ok(())
    }

    fn find_solutions_dfs<'a>(&'a self, mut cube: Cube333, depth: usize, niss_available: bool, prev: Option<Turn333>, prev_inv: Option<Turn333>, cancel_token: &'a CancelToken) -> Box<dyn Iterator<Item = Algorithm> + 'a> {
        if cancel_token.is_cancelled() {
            return Box::new(vec![].into_iter());
        }
        let lower_bound = self.step.heuristic(&cube, niss_available);
        if depth == 0 && lower_bound == 0 {
            return Box::new(vec![Algorithm::new()].into_iter());
        } else if lower_bound == 0 || lower_bound > depth {
            return Box::new(vec![].into_iter());
        }
        let values: Box<dyn Iterator<Item = Algorithm>> = Box::new(self.step.get_moveset(&cube, depth).get_allowed_moves(prev, depth)
            .flat_map(move |(turn, can_invert)|{
                cube.turn(turn);
                let normal_results = self.find_solutions_dfs(cube, depth - 1, niss_available, Some(turn), prev_inv, cancel_token)
                    .map(move |mut alg|{
                        alg.normal_moves.push(turn);
                        alg
                    });
                let results: Box<dyn Iterator<Item = Algorithm>> = if niss_available && can_invert && depth > 1 {
                    let mut cube = cube.clone();
                    cube.invert();
                    let inverse_results = self.find_solutions_dfs(cube, depth - 1, false, prev_inv, Some(turn), cancel_token)
                        .map(move |mut alg|{
                            alg.inverse_moves.push(turn);
                            Algorithm {
                                normal_moves: alg.inverse_moves,
                                inverse_moves: alg.normal_moves,
                            }
                        });
                    Box::new(normal_results.chain(inverse_results))
                } else {
                    Box::new(normal_results)
                };
                cube.turn(turn.invert());
                results
            }));
        values
    }
}

impl <S: Step + Send + 'static> ToWorker for S {
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send> {
        let cancel_token = Arc::new(CancelToken::default());
        Box::new(StepWorker {
            join_handle: None,
            cancel_token: cancel_token.clone(),
            step_runner: ThreadState::PreStart(StepIORunner {
                rc,
                tx,
                input: vec![],
                dfs_parameters: self.get_dfs_parameters(),
                current_length: 0,
                current_position: 0,
                step: self,
                cancel_token: cancel_token.clone(),
                cube_state
            }),
        })
    }
}

pub trait Step: PreStepCheck + PostStepCheck {
    fn get_dfs_parameters(&self) -> DFSParameters;
    fn get_moveset(&self, state: &Cube333, depth_left: usize) -> &'_ MoveSet;
    fn heuristic(&self, state: &Cube333, can_niss_switch: bool) -> usize;
    fn pre_step_trans(&self) -> &'_ Vec<Transformation333>;
    fn get_name(&self) -> (StepKind, String);
}

#[derive(Clone, Copy, Debug)]
pub struct DFSParameters {
    pub niss_type: NissSwitchType,
    pub min_moves: usize,
    pub max_moves: usize,
    pub absolute_min_moves: Option<usize>,
    pub absolute_max_moves: Option<usize>,
}

pub struct MoveSet {
    pub st_moves: &'static [Turn333],
    pub aux_moves: &'static [Turn333],
    pub transitions: [[bool; 18]; 18],
}

impl MoveSet {
    pub const fn new(st_moves: &'static [Turn333], aux_moves: &'static [Turn333]) -> Self {
        Self {
            st_moves,
            aux_moves,
            transitions: Self::new_default_transitions_qt_st(st_moves),
        }
    }

    // Assuming that the state change moves are [F, F', B, B'], this makes sure that allowed order is
    // B F2 instead of F2 B.
    pub const fn new_default_transitions_qt_st(st_moves: &[Turn333]) -> [[bool; 18]; 18] {
        let mut transitions = Self::new_default_transitions();
        let mut idx = 0;
        while idx < st_moves.len() {
            let turn = st_moves[idx];
            let other = Turn333::new(turn.face.opposite(), Direction::Half);
            transitions[turn.to_id()][other.to_id()] = true;
            transitions[other.to_id()][turn.to_id()] = false;
            idx += 1;
        }
        transitions
    }

    pub const fn new_default_transitions() -> [[bool; 18]; 18] {
        let mut transitions = [[true; 18]; 18];
        let dirs = [Direction::Clockwise, Direction::CounterClockwise, Direction::Half];
        let priority_faces = [CubeFace::Up, CubeFace::Front, CubeFace::Left];
        let mut idx_first = 0;
        while idx_first < dirs.len() {
            let dir_first = dirs[idx_first];
            let mut idx_last = 0;
            while idx_last < dirs.len() {
                let dir_last = dirs[idx_last];
                let mut idx = 0;
                while idx < priority_faces.len() {
                    let face = priority_faces[idx];
                    let opposite = face.opposite();
                    transitions[Turn333::new(face, dir_first).to_id()][Turn333::new(opposite, dir_last).to_id()] = false;
                    idx += 1;
                }
                let mut idx = 0;
                while idx < CubeFace::ALL.len() {
                    let face = CubeFace::ALL[idx];
                    transitions[Turn333::new(face, dir_first).to_id()][Turn333::new(face, dir_last).to_id()] = false;
                    idx += 1;
                }
                idx_last += 1;
            }
            idx_first += 1;
        }
        transitions
    }

    pub fn get_allowed_moves(&self, previous: Option<Turn333>, depth_left: usize) -> impl Iterator<Item = (Turn333, bool)> + use<'_> {
        self.st_moves.iter().cloned().map(|t|(t, true))
            .chain(self.aux_moves.iter().cloned().map(|t|(t, false)))
            .filter(move |t|t.1 || depth_left > 1)
            .filter(move |t|previous.map_or(true, |tp|self.transitions[tp.to_id()][t.0.to_id()]))
    }
}
