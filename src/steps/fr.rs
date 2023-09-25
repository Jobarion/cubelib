use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, FACES, Move, Transformation};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{DefaultPruningTableStep, IsReadyForStep, Step, StepVariant};
use itertools::Itertools;
use std::fmt::{Debug};
use crate::coords;
use crate::coords::coord::Coord;
use crate::coords::fr::{FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::coords::htr::PureHTRDRUDCoord;

pub const FR_UD_STATE_CHANGE_MOVES: [Move; 2] = [
    Move(Up, Half),
    Move(Down, Half),
];

pub const FR_UD_AUX_MOVES: [Move; 4] = [
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const FR_UD_MOVESET: MoveSet<2, 4> = MoveSet {
    st_moves: FR_UD_STATE_CHANGE_MOVES,
    aux_moves: FR_UD_AUX_MOVES,
    transitions: fr_transitions(Up),
};

pub fn fr_no_slice_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ coords::fr::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord>,
) -> Step<'a, 2, 4, C>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fr_no_slice(table, [Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ coords::fr::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord>,
) -> Step<'a, 2, 4, C>
    where
        FRUDWithSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fr(table, [Axis::UD, Axis::FB, Axis::LR])
}

pub fn fr_no_slice<'a, C: 'a + Debug, const FRA: usize>(
    table: &'a PruningTable<{ coords::fr::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord>,
    fr_axis: [Axis; FRA],
) -> Step<'a, 2, 4, C>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<2, 4, C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![], table, "fr-ud-ls"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "fr-fb-ls"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_NO_SLICE_SIZE}, FRUDNoSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "fr-lr-ls"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "fr")
}

pub fn fr<'a, C: 'a + Debug, const FRA: usize>(
    table: &'a PruningTable<{ coords::fr::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord>,
    fr_axis: [Axis; FRA],
) -> Step<'a, 2, 4, C>
    where
        FRUDWithSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    let step_variants = fr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<2, 4, C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![], table, "fr-ud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "fr-fb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<2, 4, {coords::fr::FRUD_WITH_SLICE_SIZE}, FRUDWithSliceCoord, {coords::htr::PURE_HTRDRUD_SIZE}, PureHTRDRUDCoord, C>::new(&FR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "fr-lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "fr")
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
