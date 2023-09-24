use crate::coord;
use crate::coord::{Coord, DRUDEOFBCoord, FRUDNoSliceCoord, HTRDRUDCoord, PureHTRDRUDCoord};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, FACES, Move, Transformation};
use crate::eo::eo_transitions;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::step::{IsReadyForStep, Step, StepVariant};
use itertools::Itertools;
use std::fmt::{Debug};

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

pub struct FRUDNoSliceStageTable<'a> {
    move_set: &'a MoveSet<2, 4>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>,
    name: &'a str,
}

pub struct FRFBNoSliceStageTable<'a> {
    move_set: &'a MoveSet<2, 4>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>,
    name: &'a str,
}

pub struct FRLRNoSliceStageTable<'a> {
    move_set: &'a MoveSet<2, 4>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>,
    name: &'a str,
}

pub fn fr_no_slice_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>,
) -> Step<'a, 2, 4, C>
where
    FRUDNoSliceCoord: for<'x> From<&'x C>,
    PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    Step::new(vec![
        Box::new(FRUDNoSliceStageTable::new(&table)),
        Box::new(FRFBNoSliceStageTable::new(&table)),
        Box::new(FRLRNoSliceStageTable::new(&table)),
    ])
}

pub fn fr_no_slice<'a, C: 'a + Debug, const FRA: usize>(
    table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>,
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
                Axis::UD => Some(Box::new(FRUDNoSliceStageTable::new(&table))),
                Axis::FB => Some(Box::new(FRFBNoSliceStageTable::new(&table))),
                Axis::LR => Some(Box::new(FRLRNoSliceStageTable::new(&table))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants)
}

impl<'a> FRUDNoSliceStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>) -> Self {
        Self {
            move_set: &FR_UD_MOVESET,
            pre_trans: vec![],
            table,
            name: "fr-ud",
        }
    }
}

impl<'a> FRFBNoSliceStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>) -> Self {
        Self {
            move_set: &FR_UD_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise)],
            table,
            name: "fr-fb",
        }
    }
}

impl<'a> FRLRNoSliceStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::FRUD_SIZE }, FRUDNoSliceCoord>) -> Self {
        Self {
            move_set: &FR_UD_MOVESET,
            pre_trans: vec![Transformation(Axis::Z, Clockwise)],
            table,
            name: "fr-lr",
        }
    }
}

impl<'a, C: Debug> StepVariant<2, 4, C> for FRUDNoSliceStageTable<'a>
where
    FRUDNoSliceCoord: for<'x> From<&'x C>,
    PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<2, 4> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = FRUDNoSliceCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C: Debug> StepVariant<2, 4, C> for FRFBNoSliceStageTable<'a>
where
    FRUDNoSliceCoord: for<'x> From<&'x C>,
    PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<2, 4> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = FRUDNoSliceCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C: Debug> StepVariant<2, 4, C> for FRLRNoSliceStageTable<'a>
where
    FRUDNoSliceCoord: for<'x> From<&'x C>,
    PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<2, 4> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = FRUDNoSliceCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> IsReadyForStep<C> for FRUDNoSliceStageTable<'a>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,

{
    fn is_cube_ready(&self, cube: &C) -> bool {
        //While that coordinate is broken, it correctly returns 0 iff we are in HTR
        PureHTRDRUDCoord::from(cube).val() == 0
    }
}

impl<'a, C> IsReadyForStep<C> for FRFBNoSliceStageTable<'a>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        PureHTRDRUDCoord::from(cube).val() == 0
    }
}

impl<'a, C> IsReadyForStep<C> for FRLRNoSliceStageTable<'a>
    where
        FRUDNoSliceCoord: for<'x> From<&'x C>,
        PureHTRDRUDCoord: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        PureHTRDRUDCoord::from(cube).val() == 0
    }
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
