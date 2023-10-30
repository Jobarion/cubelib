use std::fmt::Debug;

use itertools::Itertools;

use crate::steps::finish::coords::{FR_FINISH_SIZE, FRUDFinishCoord};
use crate::steps::fr::coords::{FRUD_NO_SLICE_SIZE, FRUD_WITH_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::cube::{Axis, FACES, Move, Transformation};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::defs::*;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::fr;
use crate::steps::step::StepConfig;
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};

pub const FRUD_FINISH_MOVESET: MoveSet = MoveSet {
    st_moves: fr::fr_config::FR_UD_AUX_MOVES,
    aux_moves: &[],
    transitions: finish_transitions(),
};

pub type FRFinishPruningTable = PruningTable<{ FR_FINISH_SIZE }, FRUDFinishCoord>;

pub fn from_step_config_fr<'a, C: 'a>(table: &'a FRFinishPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDWithSliceCoord: for<'x> From<&'x C>,
{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "finishud" | "finud" | "ud" => Ok(Axis::UD),
            "finishfb" | "finfb" | "fb" => Ok(Axis::FB),
            "finishlr" | "finlr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        fr_finish(table, axis?)
    } else {
        fr_finish_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.niss.unwrap_or(NissSwitchType::Never),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn from_step_config_fr_leave_slice<'a, C: 'a>(table: &'a FRFinishPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDNoSliceCoord: for<'x> From<&'x C>,
{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "finishud" | "finud" | "ud" => Ok(Axis::UD),
            "finishfb" | "finfb" | "fb" => Ok(Axis::FB),
            "finishlr" | "finlr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        fr_finish_leave_slice(table, axis?)
    } else {
        fr_finish_leave_slice_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(10),
        config.niss.unwrap_or(NissSwitchType::Never),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn fr_finish_any<'a, C: 'a>(
    table: &'a FRFinishPruningTable,
) -> Step<'a, C>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDWithSliceCoord: for<'x> From<&'x C>,
{
    fr_finish(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_finish<'a, C: 'a>(
    table: &'a FRFinishPruningTable,
    fr_axis: Vec<Axis>,
) -> Step<'a, C>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDWithSliceCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![], table, AnyPostStepCheck, ""))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, AnyPostStepCheck, ""))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, AnyPostStepCheck, ""))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FIN, true)
}


pub fn fr_finish_leave_slice_any<'a, C: 'a>(
    table: &'a FRFinishPruningTable,
) -> Step<'a, C>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDNoSliceCoord: for<'x> From<&'x C>,
{
    fr_finish_leave_slice(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_finish_leave_slice<'a, C: 'a>(
    table: &'a FRFinishPruningTable,
    fr_axis: Vec<Axis>,
) -> Step<'a, C>
    where
        FRUDFinishCoord: for<'x> From<&'x C>,
        FRUDNoSliceCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{ FR_FINISH_SIZE }, FRUDFinishCoord, {FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C, AnyPostStepCheck>::new(&FRUD_FINISH_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::FIN, true)
}

const fn finish_transitions() -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable::ANY;
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
