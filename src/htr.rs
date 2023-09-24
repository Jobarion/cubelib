use crate::coord;
use crate::coord::{Coord, DRUDEOFBCoord, HTRDRUDCoord};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation};
use crate::eo::eo_transitions;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::step::{IsReadyForStep, Step, StepVariant};
use itertools::Itertools;
use std::fmt::{Debug};

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

pub struct HTRDRUDStageTable<'a> {
    move_set: &'a MoveSet<4, 6>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>,
    name: &'a str,
}

pub struct HTRDRFBStageTable<'a> {
    move_set: &'a MoveSet<4, 6>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>,
    name: &'a str,
}

pub struct HTRDRLRStageTable<'a> {
    move_set: &'a MoveSet<4, 6>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>,
    name: &'a str,
}

pub fn htr_any<'a, C: 'a + Debug>(
    table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>,
) -> Step<'a, 4, 6, C>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    Step::new(vec![
        Box::new(HTRDRUDStageTable::new(&table)),
        Box::new(HTRDRFBStageTable::new(&table)),
        Box::new(HTRDRLRStageTable::new(&table)),
    ])
}

pub fn htr<'a, C: 'a + Debug, const DRA: usize>(
    table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>,
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
                Axis::UD => Some(Box::new(HTRDRUDStageTable::new(&table))),
                Axis::FB => Some(Box::new(HTRDRFBStageTable::new(&table))),
                Axis::LR => Some(Box::new(HTRDRLRStageTable::new(&table))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants)
}

impl<'a> HTRDRUDStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>) -> Self {
        Self {
            move_set: &HTR_DR_UD_MOVESET,
            pre_trans: vec![],
            table,
            name: "htr-drud",
        }
    }
}

impl<'a> HTRDRFBStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>) -> Self {
        Self {
            move_set: &HTR_DR_UD_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise)],
            table,
            name: "htr-drfb",
        }
    }
}

impl<'a> HTRDRLRStageTable<'a> {
    pub fn new(table: &'a PruningTable<{ coord::HTRDRUD_SIZE }, HTRDRUDCoord>) -> Self {
        Self {
            move_set: &HTR_DR_UD_MOVESET,
            pre_trans: vec![Transformation(Axis::Z, Clockwise)],
            table,
            name: "htr-drlr",
        }
    }
}

impl<'a, C: Debug> StepVariant<4, 6, C> for HTRDRUDStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 6> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = HTRDRUDCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> StepVariant<4, 6, C> for HTRDRFBStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 6> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = HTRDRUDCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> StepVariant<4, 6, C> for HTRDRLRStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 6> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = HTRDRUDCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> IsReadyForStep<C> for HTRDRUDStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        DRUDEOFBCoord::from(cube).val() == 0
    }
}

impl<'a, C> IsReadyForStep<C> for HTRDRFBStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        DRUDEOFBCoord::from(cube).val() == 0
    }
}

impl<'a, C> IsReadyForStep<C> for HTRDRLRStageTable<'a>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        DRUDEOFBCoord::from(cube).val() == 0
    }
}

const fn htr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    eo_transitions(axis_face)
}
