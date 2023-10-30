use std::fmt::Debug;

use itertools::Itertools;

use crate::steps::fr::coords::{FRUD_NO_SLICE_SIZE, FRUD_WITH_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::steps::htr::coords::{PURE_HTRDRUD_SIZE, PureHTRDRUDCoord};
use crate::cube::{Axis, Face, FACES, Move, Transformation};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::defs::*;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub const FR_UD_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Up, Half),
    Move(Down, Half),
];

pub const FR_UD_AUX_MOVES: &[Move] = &[
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const FR_UD_MOVESET: MoveSet = MoveSet {
    st_moves: FR_UD_STATE_CHANGE_MOVES,
    aux_moves: FR_UD_AUX_MOVES,
    transitions: fr_transitions(Up),
};

pub type FRLeaveSlicePruningTable = PruningTable<{ FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord>;
pub type FRPruningTable = PruningTable<{ FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord>;

pub fn from_step_config<'a, C: 'a>(table: &'a FRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        FRUDWithSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "frud" | "ud" => Ok(Axis::UD),
            "frfb" | "fb" => Ok(Axis::FB),
            "frlr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid FR substep {x}"))
        }).collect();
        fr(table, axis?)
    } else {
        fr_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn from_step_config_no_slice<'a, C: 'a>(table: &'a FRLeaveSlicePruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "frud" | "ud" => Ok(Axis::UD),
            "frfb" | "fb" => Ok(Axis::FB),
            "frlr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid FRLS substep {x}"))
        }).collect();
        fr_no_slice(table, axis?)
    } else {
        fr_no_slice_any(table)
    };

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn fr_no_slice_any<'a, C: 'a>(
    table: &'a FRLeaveSlicePruningTable,
) -> Step<'a, C>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fr_no_slice(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_any<'a, C: 'a>(
    table: &'a FRPruningTable,
) -> Step<'a, C>
    where
        FRUDWithSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fr(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_no_slice<'a, C: 'a>(
    table: &'a FRLeaveSlicePruningTable,
    fr_axis: Vec<Axis>,
) -> Step<'a, C>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FRLS, true)
}

pub fn fr<'a, C: 'a>(
    table: &'a FRPruningTable,
    fr_axis: Vec<Axis>,
) -> Step<'a, C>
    where
        FRUDWithSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C, AnyPostStepCheck>::new(&FR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FR, true)
}

const fn fr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable::moves_to_mask([
        Move(axis_face, Half),
        Move(axis_face.opposite(), Half),
    ]);
    while i < FACES.len() {
        transitions[Move(FACES[i], Clockwise).to_id()] = TransitionTable::new(
            TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize],
            can_end_mask,
        );
        transitions[Move(FACES[i], Half).to_id()] = TransitionTable::new(
            TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize],
            can_end_mask,
        );
        transitions[Move(FACES[i], CounterClockwise).to_id()] = TransitionTable::new(
            TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize],
            can_end_mask,
        );
        i += 1;
    }
    transitions
}
