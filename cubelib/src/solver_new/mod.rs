use crate::algs::Algorithm;
use crate::cube::{Cube333, Transformation333};
use crate::defs::{StepKind, StepVariant};
use crate::solver_new::dr::{DRBuilder, RZPBuilder, RZPStep};
use crate::solver_new::eo::EOBuilder;
use crate::solver_new::finish::{DRFinishBuilder, FRFinishBuilder, HTRFinishBuilder};
use crate::solver_new::fr::FRBuilder;
use crate::solver_new::group::StepGroup;
use crate::solver_new::htr::HTRBuilder;
use crate::solver_new::step::{DFSParameters, MoveSet};
use crate::solver_new::thread_util::{ToWorker, Worker};
use crate::solver_new::util_steps::FilterExcluded;
use crate::steps::step::{PostStepCheck, PreStepCheck, StepConfig};
use std::str::FromStr;

pub mod ar;
pub mod dr;
pub mod eo;
pub mod finish;
pub mod fr;
pub mod group;
pub mod htr;
pub mod step;
pub mod thread_util;
pub mod util_cube;
pub mod util_steps;

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
    fn get_variant(&self) -> StepVariant;
}

pub fn build_steps(mut steps: Vec<StepConfig>) -> Result<StepGroup, String> {
    let mut step_groups = vec![];
    let mut previous = None;
    steps.reverse();
    while !steps.is_empty() {
        let mut step = steps.pop().unwrap();
        let mut next_prev = Some(step.kind.clone());
        let excluded = step.excluded.clone();
        let mut step_group = match (previous, step.kind.clone()) {
            (None, StepKind::EO) => EOBuilder::try_from(step)
                .map_err(|_| "Failed to parse EO step")?
                .build(),
            (Some(StepKind::EO), StepKind::RZP) => {
                let mut dr = steps.pop().ok_or("Expected DR to follow RZP".to_string())?;
                next_prev = Some(StepKind::DR);
                let rzp_builder =
                    RZPBuilder::try_from(step).map_err(|_| "Failed to parse RZP step")?;
                let triggers = dr
                    .params
                    .remove("triggers")
                    .ok_or("Found RZP, but DR step has no triggers".to_string())?;
                let dr_excluded = dr.excluded.clone();
                let mut dr_step = DRBuilder::try_from(dr)
                    .map_err(|_| "Failed to parse DR step")?
                    .triggers(
                        triggers
                            .split(",")
                            .map(Algorithm::from_str)
                            .collect::<Result<_, _>>()
                            .map_err(|_| "Unable to parse algorithm")?,
                    )
                    .rzp(rzp_builder)
                    .build();
                if !dr_excluded.is_empty() {
                    dr_step.with_predicates(vec![FilterExcluded::new(dr_excluded)]);
                }
                dr_step
            }
            (Some(StepKind::EO), StepKind::DR) => match step.params.remove("triggers") {
                None => DRBuilder::try_from(step)
                    .map_err(|_| "Failed to parse DR step")?
                    .build(),
                Some(triggers) => {
                    let rzp = RZPStep::builder()
                        .max_length(step.max.unwrap_or(3).min(3) as usize)
                        .max_absolute_length(step.absolute_max.unwrap_or(6).min(6) as usize);
                    DRBuilder::try_from(step)
                        .map_err(|_| "Failed to parse DR step")?
                        .triggers(
                            triggers
                                .split(",")
                                .map(Algorithm::from_str)
                                .collect::<Result<_, _>>()
                                .map_err(|_| "Unable to parse algorithm")?,
                        )
                        .rzp(rzp)
                        .build()
                }
            },
            (Some(StepKind::DR), StepKind::HTR) => HTRBuilder::try_from(step)
                .map_err(|_| "Failed to parse HTR step")?
                .build(),
            (Some(StepKind::HTR), StepKind::FR) | (Some(StepKind::HTR), StepKind::FRLS) => {
                FRBuilder::try_from(step)
                    .map_err(|_| "Failed to parse FR step")?
                    .build()
            }
            (Some(StepKind::FR), StepKind::FIN) => FRFinishBuilder::try_from(step)
                .map_err(|_| "Failed to parse FIN step")?
                .build(),
            (Some(StepKind::FRLS), StepKind::FINLS) => FRFinishBuilder::try_from(step)
                .map_err(|_| "Failed to parse FIN step")?
                .build(),
            (Some(StepKind::HTR), StepKind::FIN) | (Some(StepKind::HTR), StepKind::FINLS) => {
                let htr_breaking = step
                    .params
                    .remove("htr-breaking")
                    .map(|x| bool::from_str(x.to_lowercase().as_str()).unwrap_or(false))
                    .unwrap_or(false);
                if htr_breaking {
                    DRFinishBuilder::try_from(step)
                        .map_err(|_| "Failed to parse FIN step")?
                        .from_htr()
                        .build()
                } else {
                    HTRFinishBuilder::try_from(step)
                        .map_err(|_| "Failed to parse FIN step")?
                        .build()
                }
            }
            (Some(StepKind::DR), StepKind::FIN) => DRFinishBuilder::try_from(step)
                .map_err(|_| "Failed to parse FIN step")?
                .build(),
            (None, x) => return Err(format!("{x:?} is not supported as a first step",)),
            (Some(a), b) => return Err(format!("Step order {a:?} > {b:?} is not supported")),
        };
        if !excluded.is_empty() {
            step_group.with_predicates(vec![FilterExcluded::new(excluded)]);
        }
        step_groups.push(step_group);
        previous = next_prev;
    }
    Ok(StepGroup::sequential(step_groups))
}
