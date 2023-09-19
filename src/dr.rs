use std::cmp::{max, min};
use std::rc::Rc;
use std::str::FromStr;
use itertools::Itertools;
use crate::algs::Algorithm;
use crate::coord::DRUDEOFBCoord;
use crate::cube::{Axis, Cube, Face, FACES, Invertible, Move, Transformation, Turn, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::CubieCube;
use crate::df_search::{ALL_MOVES, dfs_iter, NissType, SearchOptions};
use crate::eo::EOCount;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::step::{IsReadyForStep, Step, StepVariant};

pub const DR_UD_EO_FB_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Right, Clockwise), Move(Right, CounterClockwise),
    Move(Left, Clockwise), Move(Left, CounterClockwise),
];

pub const DR_UD_EO_FB_MOVES: [Move; 10] = [
    Move(Up, Clockwise), Move(Up, CounterClockwise), Move(Up, Half),
    Move(Down, Clockwise), Move(Down, CounterClockwise), Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const DR_UD_EO_FB_MOVESET: MoveSet<4 , 10> = MoveSet {
    st_moves: DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: DR_UD_EO_FB_MOVES,
    transitions: dr_transitions(Left)
};

pub struct DREOUDStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<1082565, DRUDEOFBCoord>,
}

pub struct DREOFBStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<1082565, DRUDEOFBCoord>,
}

pub struct DREOLRStageTable<'a> {
    move_set: &'a MoveSet<4, 10>,
    pre_trans: Vec<Transformation>,
    table: &'a PruningTable<1082565, DRUDEOFBCoord>,
}

pub fn dr<'a, C: EOCount + 'a, const EOA: usize, const DRA: usize>(table: &'a PruningTable<1082565, DRUDEOFBCoord>, eo_axis: [Axis; EOA], dr_axis: [Axis; DRA]) -> Step<'a, 4, 10, C> where DRUDEOFBCoord: for<'x> From<&'x C> {
    let step_variants = eo_axis.into_iter()
        .flat_map(|eo| dr_axis.into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<4, 10, C> + 'a>> = match x {
                (Axis::UD, Axis::FB) => Some(Box::new(DREOUDStageTable::new_drfb_eoud(&table))),
                (Axis::UD, Axis::LR) => Some(Box::new(DREOUDStageTable::new_drlr_eoud(&table))),
                (Axis::FB, Axis::UD) => Some(Box::new(DREOFBStageTable::new_drud_eofb(&table))),
                (Axis::FB, Axis::LR) => Some(Box::new(DREOFBStageTable::new_drud_eofb(&table))),
                (Axis::LR, Axis::UD) => Some(Box::new(DREOLRStageTable::new_drud_eolr(&table))),
                (Axis::LR, Axis::FB) => Some(Box::new(DREOLRStageTable::new_drfb_eolr(&table))),
                (eo, dr) => None
            };
            x
        })
        .collect_vec();
    Step::new(step_variants)
}

pub fn dr_any<'a, C: EOCount + 'a>(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Step<'a, 4, 10, C> where DRUDEOFBCoord: for<'x> From<&'x C> {
    Step::new(vec![
        Box::new(DREOUDStageTable::new_drfb_eoud(&table)),
        Box::new(DREOUDStageTable::new_drlr_eoud(&table)),
        Box::new(DREOFBStageTable::new_drud_eofb(&table)),
        Box::new(DREOFBStageTable::new_drlr_eofb(&table)),
        Box::new(DREOLRStageTable::new_drfb_eolr(&table)),
        Box::new(DREOLRStageTable::new_drud_eolr(&table)),
    ])
}

impl <'a> DREOFBStageTable<'a> {
    pub fn new_drud_eofb(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![],
            table
        }
    }

    pub fn new_drlr_eofb(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Z, Clockwise)],
            table
        }
    }
}

impl <'a> DREOUDStageTable<'a> {
    pub fn new_drfb_eoud(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise)],
            table
        }
    }
    pub fn new_drlr_eoud(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise), Transformation(Axis::Z, Clockwise)],
            table
        }
    }
}

impl <'a> DREOLRStageTable<'a> {
    pub fn new_drud_eolr(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Y, Clockwise)],
            table
        }
    }
    pub fn new_drfb_eolr(table: &'a PruningTable<1082565, DRUDEOFBCoord>) -> Self {
        Self {
            move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Y, Clockwise), Transformation(Axis::Z, Clockwise)],
            table
        }
    }
}

impl <'a, C: EOCount> StepVariant<4, 10, C> for DREOUDStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
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
}

impl <'a, C: EOCount> StepVariant<4, 10, C> for DREOFBStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
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
}

impl <'a, C: EOCount> StepVariant<4, 10, C> for DREOLRStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
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
}

impl <'a, C: EOCount> IsReadyForStep<C> for DREOUDStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
    fn is_cube_ready(&self, cube: &C) -> bool {
        cube.count_bad_edges().0 == 0
    }
}

impl <'a, C: EOCount> IsReadyForStep<C> for DREOFBStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
    fn is_cube_ready(&self, cube: &C) -> bool {
        cube.count_bad_edges().0 == 1
    }
}

impl <'a, C: EOCount> IsReadyForStep<C> for DREOLRStageTable<'a> where DRUDEOFBCoord: for<'x> From<&'x C> {
    fn is_cube_ready(&self, cube: &C) -> bool {
        cube.count_bad_edges().0 == 2
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
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    while i < FACES.len() {
        transitions[Move(FACES[i], Clockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], CounterClockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        i += 1;
    }
    i = 0;
    transitions[Move(axis_face, Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face as usize], TransitionTable::except_moves_to_mask([Move(axis_face.opposite(), Clockwise), Move(axis_face.opposite(), CounterClockwise)]));
    transitions[Move(axis_face.opposite(), Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face.opposite() as usize], TransitionTable::except_moves_to_mask([Move(axis_face, Clockwise), Move(axis_face,CounterClockwise)]));
    transitions
}