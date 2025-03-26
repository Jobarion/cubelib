use std::str::FromStr;
use crate::algs::Algorithm;
use crate::cube::{Cube333, Transformation333};
use crate::defs::StepKind;
use crate::solver_new::dr::{DRBuilder, RZPBuilder, RZPStep};
use crate::solver_new::eo::EOBuilder;
use crate::solver_new::finish::{FRFinishBuilder, HTRFinishBuilder};
use crate::solver_new::fr::FRBuilder;
use crate::solver_new::group::StepGroup;
use crate::solver_new::htr::HTRBuilder;
use crate::solver_new::step::{DFSParameters, MoveSet};
use crate::solver_new::thread_util::{ToWorker, Worker};
use crate::steps::step::{PostStepCheck, PreStepCheck, StepConfig};

pub mod step;
pub mod eo;
pub mod dr;
pub mod group;
pub mod thread_util;
pub mod util_steps;
pub mod htr;
pub mod util_cube;
pub mod fr;
pub mod finish;
pub mod ar;

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

pub fn build_steps(mut steps: Vec<StepConfig>) -> Result<StepGroup, String> {
    let mut step_groups = vec![];
    let mut previous = None;
    steps.reverse();
    while !steps.is_empty() {
        let mut step = steps.pop().unwrap();
        let mut next_prev = Some(step.kind.clone());
        step_groups.push(match (previous, step.kind.clone()) {
            (None, StepKind::EO) => EOBuilder::try_from(step).map_err(|_|"Failed to parse EO step")?.build(),
            (Some(StepKind::EO), StepKind::RZP) => {
                let mut dr = steps.pop().ok_or("Expected DR to follow RZP".to_string())?;
                next_prev = Some(StepKind::DR);
                let rzp_builder = RZPBuilder::try_from(step).map_err(|_|"Failed to parse RZP step")?;
                let triggers = dr.params.remove("triggers").ok_or("Found RZP, but DR step has no triggers".to_string())?;
                DRBuilder::try_from(dr).map_err(|_|"Failed to parse DR step")?
                    .triggers(triggers.split(",")
                        .map(Algorithm::from_str)
                        .collect::<Result<_, _>>()
                        .map_err(|_|"Unable to parse algorithm")?)
                    .rzp(rzp_builder)
                    .build()
            },
            (Some(StepKind::EO), StepKind::DR) => {
                match step.params.remove("triggers") {
                    None => DRBuilder::try_from(step).map_err(|_|"Failed to parse DR step")?.build(),
                    Some(triggers) => {
                        let rzp = RZPStep::builder()
                            .max_length(step.max.unwrap_or(3).min(3) as usize)
                            .max_absolute_length(step.absolute_max.unwrap_or(6).min(6) as usize);
                        DRBuilder::try_from(step).map_err(|_|"Failed to parse DR step")?
                            .triggers(triggers.split(",")
                                .map(Algorithm::from_str)
                                .collect::<Result<_, _>>()
                                .map_err(|_|"Unable to parse algorithm")?)
                            .rzp(rzp)
                            .build()
                    }
                }
            },
            (Some(StepKind::DR), StepKind::HTR) => HTRBuilder::try_from(step).map_err(|_|"Failed to parse HTR step")?.build(),
            (Some(StepKind::HTR), StepKind::FR) | (Some(StepKind::HTR), StepKind::FRLS)  => FRBuilder::try_from(step).map_err(|_|"Failed to parse FR step")?.build(),
            (Some(StepKind::FR), StepKind::FIN) => FRFinishBuilder::try_from(step).map_err(|_|"Failed to parse FIN step")?.build(),
            (Some(StepKind::FRLS), StepKind::FINLS) => FRFinishBuilder::try_from(step).map_err(|_|"Failed to parse FIN step")?.build(),
            (Some(StepKind::HTR), StepKind::FIN) | (Some(StepKind::HTR), StepKind::FINLS) => HTRFinishBuilder::try_from(step).map_err(|_|"Failed to parse FIR step")?.build(),
            (None, x) => return Err(format!("{x:?} is not supported as a first step", )),
            (Some(a), b) => return Err(format!("Step order {a:?} > {b:?} is not supported")),
        });
        previous = next_prev;
    }
    Ok(StepGroup::sequential(step_groups))
}