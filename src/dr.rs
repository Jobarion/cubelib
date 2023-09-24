use crate::coord;
use itertools::Itertools;

use crate::coord::{DRUDEOFBCoord, EOCoordFB};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation};
use crate::eo::{eo_transitions};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::step::{IsReadyForStep, Step, StepVariant};

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

pub struct DREOUDStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
    name: &'a str,
}

pub struct DREOFBStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
    name: &'a str,
}

pub struct DREOLRStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
    name: &'a str,
}

pub fn dr<'a, C: 'a, const EOA: usize, const DRA: usize>(
    table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
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
                (Axis::UD, Axis::FB) => Some(Box::new(DREOUDStageTable::new_drfb_eoud(&table))),
                (Axis::UD, Axis::LR) => Some(Box::new(DREOUDStageTable::new_drlr_eoud(&table))),
                (Axis::FB, Axis::UD) => Some(Box::new(DREOFBStageTable::new_drud_eofb(&table))),
                (Axis::FB, Axis::LR) => Some(Box::new(DREOFBStageTable::new_drud_eofb(&table))),
                (Axis::LR, Axis::UD) => Some(Box::new(DREOLRStageTable::new_drud_eolr(&table))),
                (Axis::LR, Axis::FB) => Some(Box::new(DREOLRStageTable::new_drfb_eolr(&table))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec();
    Step::new(step_variants)
}

pub fn dr_any<'a, C: 'a>(
    table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>,
) -> Step<'a, 4, 10, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    Step::new(vec![
        Box::new(DREOUDStageTable::new_drfb_eoud(&table)),
        Box::new(DREOUDStageTable::new_drlr_eoud(&table)),
        Box::new(DREOFBStageTable::new_drud_eofb(&table)),
        Box::new(DREOFBStageTable::new_drlr_eofb(&table)),
        Box::new(DREOLRStageTable::new_drfb_eolr(&table)),
        Box::new(DREOLRStageTable::new_drud_eolr(&table)),
    ])
}

impl<'a> DREOFBStageTable<'a> {
    pub fn new_drud_eofb(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![],
            table,
            name: "drud-eofb",
        }
    }

    pub fn new_drlr_eofb(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Z, Clockwise)],
            table,
            name: "drlr-eofb",
        }
    }
}

impl<'a> DREOUDStageTable<'a> {
    pub fn new_drfb_eoud(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise)],
            table,
            name: "drfb-eoud",
        }
    }
    pub fn new_drlr_eoud(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![
                Transformation(Axis::X, Clockwise),
                Transformation(Axis::Z, Clockwise),
            ],
            table,
            name: "drlf-eoud",
        }
    }
}

impl<'a> DREOLRStageTable<'a> {
    pub fn new_drud_eolr(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Y, Clockwise)],
            table,
            name: "drud-eolr",
        }
    }
    pub fn new_drfb_eolr(table: &'a PruningTable<{ coord::DRUDEOFB_SIZE }, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![
                Transformation(Axis::Y, Clockwise),
                Transformation(Axis::Z, Clockwise),
            ],
            table,
            name: "drfb-eolr",
        }
    }
}

impl<'a, C> StepVariant<4, 10, C> for DREOUDStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 10> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = DRUDEOFBCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> StepVariant<4, 10, C> for DREOFBStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 10> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = DRUDEOFBCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> StepVariant<4, 10, C> for DREOLRStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet<4, 10> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = DRUDEOFBCoord::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl<'a, C> IsReadyForStep<C> for DREOUDStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        EOCoordFB::from(cube).0 == 0
    }
}

impl<'a, C> IsReadyForStep<C> for DREOFBStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        EOCoordFB::from(cube).0 == 0
    }
}

impl<'a, C> IsReadyForStep<C> for DREOLRStageTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        EOCoordFB::from(cube).0 == 0
    }
}

// impl <'a> DRStageTable<'a, 1> {
//     pub fn new_drud_eorl(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
//         DRStageTable {
//             move_set: &DR_UD_EO_FB_MOVESET,
//             pre_trans: &[Transformation(Axis::Y, Clockwise)],
//             table
//         }
//     }
// }

// pub fn drud_eofb_stage() -> StageOptions<4, 10, 0> {
//     StageOptions { move_set: DR_UD_EO_FB_MOVESET, transformations: [] }
// }

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    eo_transitions(axis_face)
}
