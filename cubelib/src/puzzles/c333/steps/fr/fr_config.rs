use itertools::Itertools;

use crate::defs::*;
use crate::solver::lookup_table::LookupTable;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{MoveSet333, Step333};
use crate::puzzles::c333::steps::fr::coords::{FRUD_NO_SLICE_SIZE, FRUD_WITH_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::puzzles::c333::steps::htr::coords::{PURE_HTRDRUD_SIZE, PureHTRDRUDCoord};
use crate::puzzles::cube::{CubeAxis, CubeFace};
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::Direction::*;
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub const FR_UD_STATE_CHANGE_MOVES: &[Turn333] = &[
    Turn333::new(Up, Half),
    Turn333::new(Down, Half),
];

pub const FR_UD_AUX_MOVES: &[Turn333] = &[
    Turn333::new(Right, Half),
    Turn333::new(Left, Half),
    Turn333::new(Front, Half),
    Turn333::new(Back, Half),
];

pub const FR_UD_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: FR_UD_STATE_CHANGE_MOVES,
    aux_moves: FR_UD_AUX_MOVES,
    transitions: &fr_transitions(Up),
};

pub type FRLeaveSlicePruningTable = LookupTable<{ FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord>;
pub type FRPruningTable = LookupTable<{ FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord>;
pub type FRLeaveSlicePruningTableStep<'a> = DefaultPruningTableStep::<'a, {FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, Turn333, Transformation333, Cube333, TransitionTable333, AnyPostStepCheck>;
pub type FRPruningTableStep<'a> = DefaultPruningTableStep::<'a, {FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, Turn333, Transformation333, Cube333, TransitionTable333, AnyPostStepCheck>;

pub fn from_step_config(table: &FRPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "frud" | "ud" => Ok(CubeAxis::UD),
            "frfb" | "fb" => Ok(CubeAxis::FB),
            "frlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid FR substep {x}"))
        }).collect();
        fr(table, axis?)
    } else {
        fr_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn from_step_config_no_slice(table: &FRLeaveSlicePruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "frud" | "ud" => Ok(CubeAxis::UD),
            "frfb" | "fb" => Ok(CubeAxis::FB),
            "frlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid FRLS substep {x}"))
        }).collect();
        fr_no_slice(table, axis?)
    } else {
        fr_no_slice_any(table)
    };

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.absolute_min,
        config.absolute_max,
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn fr_no_slice_any(table: &FRLeaveSlicePruningTable) -> Step333 {
    fr_no_slice(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn fr_any(table: &FRPruningTable) -> Step333 {
    fr(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn fr_no_slice<'a>(table: &'a FRLeaveSlicePruningTable, fr_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(FRLeaveSlicePruningTableStep::new(&FR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                CubeAxis::FB => Some(Box::new(FRLeaveSlicePruningTableStep::new(&FR_UD_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                CubeAxis::LR => Some(Box::new(FRLeaveSlicePruningTableStep::new(&FR_UD_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FRLS, true)
}

pub fn fr<'a>(table: &'a FRPruningTable, fr_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(FRPruningTableStep::new(&FR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                CubeAxis::FB => Some(Box::new(FRPruningTableStep::new(&FR_UD_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                CubeAxis::LR => Some(Box::new(FRPruningTableStep::new(&FR_UD_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FR, true)
}

const fn fr_transitions(axis_face: CubeFace) -> [TransitionTable333; 18] {
    let mut transitions = [TransitionTable333::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable333::moves_to_mask([
        Turn333::new(axis_face, Half),
        Turn333::new(axis_face.opposite(), Half),
    ]);
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
