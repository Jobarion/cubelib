use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{DefaultPruningTableStep, IsReadyForStep, Step, StepVariant};
use itertools::Itertools;
use std::fmt::{Debug};
use crate::coords;
use crate::coords::coord::Coord;
use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::htr::HTRDRUDCoord;

pub const HTR_DR_UD_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
];

pub const HTR_MOVES: [Move; 6] = [
    Move(Up, Half),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const HTR_DR_UD_MOVESET: MoveSet<4, 6> = MoveSet {
    st_moves: HTR_DR_UD_STATE_CHANGE_MOVES,
    aux_moves: HTR_MOVES,
    transitions: htr_transitions(Up),
};

pub fn htr_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ coords::htr::HTRDRUD_SIZE }, HTRDRUDCoord>,
) -> Step<'a, 4, 6, C>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    htr(table, [Axis::UD, Axis::FB, Axis::LR])
}

pub fn htr<'a, C: 'a + Debug, const DRA: usize>(
    table: &'a PruningTable<{ coords::htr::HTRDRUD_SIZE }, HTRDRUDCoord>,
    dr_axis: [Axis; DRA],
) -> Step<'a, 4, 6, C>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    let step_variants = dr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<4, 6, C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<4, 6, {coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![], table, "htr-drud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<4, 6, {coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "htr-drfb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<4, 6, {coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "htr-drlr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "htr")
}

const fn htr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_transitions(axis_face)
}
