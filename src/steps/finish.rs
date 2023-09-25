use std::fmt::Debug;

use itertools::Itertools;

use crate::coords;
use crate::coords::finish::{FR_FINISH_SIZE, FRFinishCoord};
use crate::coords::fr::{FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::coords::htr::PureHTRDRUDCoord;
use crate::cube::{Axis, FACES, Move, Transformation};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::fr;
use crate::steps::step::{DefaultPruningTableStep, Step, StepVariant};

pub const FR_FINISH_MOVESET: MoveSet<4, 0> = MoveSet {
    st_moves: fr::FR_UD_AUX_MOVES,
    aux_moves: [],
    transitions: finish_transitions(),
};

pub fn fr_finish_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ FR_FINISH_SIZE }, FRFinishCoord>,
) -> Step<'a, 4, 0, C>
    where
        FRFinishCoord: for<'x> From<&'x C>,
        FRUDWithSliceCoord: for<'x> From<&'x C>,
{
    fr_finish(table, [Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_finish<'a, C: 'a + Debug, const FRA: usize>(
    table: &'a PruningTable<{ FR_FINISH_SIZE }, FRFinishCoord>,
    fr_axis: [Axis; FRA],
) -> Step<'a, 4, 0, C>
    where
        FRFinishCoord: for<'x> From<&'x C>,
        FRUDWithSliceCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<4, 0, C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![], table, "finish"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "finish"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "finish"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "finish")
}


pub fn fr_finish_leave_slice_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ FR_FINISH_SIZE }, FRFinishCoord>,
) -> Step<'a, 4, 0, C>
    where
        FRFinishCoord: for<'x> From<&'x C>,
        FRUDNoSliceCoord: for<'x> From<&'x C>,
{
    fr_leave_slice_finish(table, [Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_leave_slice_finish<'a, C: 'a + Debug, const FRA: usize>(
    table: &'a PruningTable<{ FR_FINISH_SIZE }, FRFinishCoord>,
    fr_axis: [Axis; FRA],
) -> Step<'a, 4, 0, C>
    where
        FRFinishCoord: for<'x> From<&'x C>,
        FRUDNoSliceCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<4, 0, C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![], table, "finish"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "finish"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<4, 0, { FR_FINISH_SIZE }, FRFinishCoord, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, C>::new(&FR_FINISH_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "finish"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "finish")
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
