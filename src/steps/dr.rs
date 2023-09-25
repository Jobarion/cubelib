use itertools::Itertools;
use crate::coords;

use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::eo::EOCoordFB;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{DefaultPruningTableStep, IsReadyForStep, Step, StepVariant};

pub const DR_UD_EO_FB_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Right, Clockwise),
    Move(Right, CounterClockwise),
    Move(Left, Clockwise),
    Move(Left, CounterClockwise),
];

pub const DR_UD_EO_FB_MOVES: [Move; 10] = [
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Up, Half),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const DR_UD_EO_FB_MOVESET: MoveSet<4, 10> = MoveSet {
    st_moves: DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: DR_UD_EO_FB_MOVES,
    transitions: dr_transitions(Left),
};

pub fn dr<'a, C: 'a, const EOA: usize, const DRA: usize>(
    table: &'a PruningTable<{ coords::dr::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
) -> Step<'a, 4, 10, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = eo_axis
        .into_iter()
        .flat_map(|eo| dr_axis.into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<4, 10, C> + 'a>> = match x {
                (Axis::UD, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X], table, "drfb-eoud"))),
                (Axis::UD, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X, Transformation::Z], table, "drlr-eoud"))),
                (Axis::FB, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![], table, "drud-eofb"))),
                (Axis::FB, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Z], table, "drlr-eofb"))),
                (Axis::LR, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y], table, "drud-eolr"))),
                (Axis::LR, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<4, 10, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y, Transformation::Z], table, "drfb-eolr"))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "dr")
}

pub fn dr_any<'a, C: 'a>(
    table: &'a PruningTable<{ coords::dr::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
) -> Step<'a, 4, 10, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    dr(table, [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR])
}

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_transitions(axis_face)
}
