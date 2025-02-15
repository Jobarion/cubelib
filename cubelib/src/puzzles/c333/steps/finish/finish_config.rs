use std::rc::Rc;
use itertools::Itertools;

use crate::defs::*;
use crate::solver::lookup_table::LookupTable;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{fr, MoveSet333, Step333};
use crate::puzzles::c333::steps::finish::coords::{FR_FINISH_SIZE, FRUDFinishCoord, HTR_FINISH_SIZE, HTR_LEAVE_SLICE_FINISH_SIZE, HTRFinishCoord, HTRLeaveSliceFinishCoord};
use crate::puzzles::c333::steps::fr::coords::{FRUD_NO_SLICE_SIZE, FRUD_WITH_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::puzzles::c333::steps::htr::coords::{PURE_HTRDRUD_SIZE, PureHTRDRUDCoord};
use crate::puzzles::cube::{CubeAxis, CubeFace};
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::Direction::*;
use crate::steps::step::{DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub const FRUD_FINISH_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: fr::fr_config::FR_UD_AUX_MOVES,
    aux_moves: &[],
    transitions: &finish_transitions(),
};

pub const HTR_FINISH_MOVESET: MoveSet333 = MoveSet333 {
    aux_moves: &[],
    st_moves: &[
        Turn333::new(Up, Half),
        Turn333::new(Down, Half),
        Turn333::new(Right, Half),
        Turn333::new(Left, Half),
        Turn333::new(Front, Half),
        Turn333::new(Back, Half),
    ],
    transitions: &finish_transitions(),
};
pub type FRFinishPruningTable = LookupTable<{ FR_FINISH_SIZE }, FRUDFinishCoord>;
pub type FRFinishPruningTableStep<'a> = DefaultPruningTableStep::<'a, { FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, Turn333, Transformation333, Cube333, TransitionTable333>;

pub type FRFinishLeaveSlicePruningTableStep<'a> = DefaultPruningTableStep::<'a, { FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, Turn333, Transformation333, Cube333, TransitionTable333>;

pub type HTRFinishPruningTable = LookupTable<{ HTR_FINISH_SIZE }, HTRFinishCoord>;
pub type HTRFinishPruningTableStep<'a> = DefaultPruningTableStep::<'a, { HTR_FINISH_SIZE }, HTRFinishCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, Turn333, Transformation333, Cube333, TransitionTable333>;

pub type HTRLeaveSliceFinishPruningTable = LookupTable<{ HTR_LEAVE_SLICE_FINISH_SIZE }, HTRLeaveSliceFinishCoord>;
pub type HTRLeaveSliceFinishPruningTableStep<'a> = DefaultPruningTableStep::<'a, { HTR_LEAVE_SLICE_FINISH_SIZE }, HTRLeaveSliceFinishCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, Turn333, Transformation333, Cube333, TransitionTable333>;

pub fn from_step_config_fr(table: &FRFinishPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "finishud" | "finud" | "ud" => Ok(CubeAxis::UD),
            "finishfb" | "finfb" | "fb" => Ok(CubeAxis::FB),
            "finishlr" | "finlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        fr_finish(table, axis?)
    } else {
        fr_finish_any(table)
    };

    if !config.params.is_empty() {
        return Err(format!("Unreognized parameters: {:?}", config.params.keys()))
    }

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        NissSwitchType::Never,
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn from_step_config_fr_leave_slice(table: &FRFinishPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "finishud" | "finud" | "ud" => Ok(CubeAxis::UD),
            "finishfb" | "finfb" | "fb" => Ok(CubeAxis::FB),
            "finishlr" | "finlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        fr_finish_leave_slice(table, axis?)
    } else {
        fr_finish_leave_slice_any(table)
    };

    if !config.params.is_empty() {
        return Err(format!("Unreognized parameters: {:?}", config.params.keys()))
    }

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        NissSwitchType::Never,
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn from_step_config_htr(table: &HTRFinishPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        NissSwitchType::Never,
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((htr_finish(table), search_opts))
}

pub fn from_step_config_htr_leave_slice(table: &HTRLeaveSliceFinishPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "finishud" | "finud" | "ud" => Ok(CubeAxis::UD),
            "finishfb" | "finfb" | "fb" => Ok(CubeAxis::FB),
            "finishlr" | "finlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid leave slice finish axis {x}"))
        }).collect();
        htr_finish_leave_slice(table, axis?)
    } else {
        htr_finish_leave_slice_any(table)
    };

    if !config.params.is_empty() {
        return Err(format!("Unreognized parameters: {:?}", config.params.keys()))
    }

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        NissSwitchType::Never,
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn fr_finish_any(table: &FRFinishPruningTable) -> Step333 {
    fr_finish(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn fr_finish<'a>(table: &'a FRFinishPruningTable, fr_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(FRFinishPruningTableStep::new(&FRUD_FINISH_MOVESET, vec![], table, Rc::new(vec![]), ""))),
                CubeAxis::FB => Some(Box::new(FRFinishPruningTableStep::new(&FRUD_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, Rc::new(vec![]), ""))),
                CubeAxis::LR => Some(Box::new(FRFinishPruningTableStep::new(&FRUD_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, Rc::new(vec![]), ""))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FIN, true)
}

pub fn htr_finish(table: &HTRFinishPruningTable) -> Step333 {
    Step::new(vec![
        Box::new(HTRFinishPruningTableStep::new(&HTR_FINISH_MOVESET, vec![], table, Rc::new(vec![]), ""))
    ], StepKind::FIN, true)
}

pub fn htr_finish_leave_slice_any(table: &HTRLeaveSliceFinishPruningTable) -> Step333 {
    htr_finish_leave_slice(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn htr_finish_leave_slice<'a>(table: &'a HTRLeaveSliceFinishPruningTable, slice_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = slice_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(HTRLeaveSliceFinishPruningTableStep::new(&HTR_FINISH_MOVESET, vec![], table, Rc::new(vec![]), "ud"))),
                CubeAxis::FB => Some(Box::new(HTRLeaveSliceFinishPruningTableStep::new(&HTR_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, Rc::new(vec![]), "fb"))),
                CubeAxis::LR => Some(Box::new(HTRLeaveSliceFinishPruningTableStep::new(&HTR_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, Rc::new(vec![]), "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FINLS, true)
}


pub fn fr_finish_leave_slice_any(table: &FRFinishPruningTable) -> Step333 {
    fr_finish_leave_slice(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn fr_finish_leave_slice<'a>(table: &'a FRFinishPruningTable, fr_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(FRFinishLeaveSlicePruningTableStep::new(&FRUD_FINISH_MOVESET, vec![], table, Rc::new(vec![]), "ud"))),
                CubeAxis::FB => Some(Box::new(FRFinishLeaveSlicePruningTableStep::new(&FRUD_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, Rc::new(vec![]), "fb"))),
                CubeAxis::LR => Some(Box::new(FRFinishLeaveSlicePruningTableStep::new(&FRUD_FINISH_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, Rc::new(vec![]), "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FIN, true)
}

const fn finish_transitions() -> [TransitionTable333; 18] {
    let mut transitions = [TransitionTable333::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable333::ANY;
    while i < CubeFace::ALL.len() {
        transitions[Turn333::new(CubeFace::ALL[i], Clockwise).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        transitions[Turn333::new(CubeFace::ALL[i], Half).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        transitions[Turn333::new(CubeFace::ALL[i], CounterClockwise).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        i += 1;
    }
    transitions
}
