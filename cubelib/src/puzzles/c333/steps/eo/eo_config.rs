use itertools::Itertools;

use crate::algs::Algorithm;
use crate::defs::*;
use crate::solver::lookup_table::PruningTable;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, EdgeCube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{MoveSet333, Step333};
use crate::puzzles::c333::steps::eo::coords::EOCoordFB;
use crate::puzzles::cube::{CubeAxis, CubeFace, CubeOuterTurn};
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::Direction::*;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

pub const UD_EO_STATE_CHANGE_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
];

pub const UD_EO_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Up, Half),
    Turn333::new(Down, Half),
    Turn333::new(Front, Clockwise),
    Turn333::new(Front, CounterClockwise),
    Turn333::new(Front, Half),
    Turn333::new(Back, Clockwise),
    Turn333::new(Back, CounterClockwise),
    Turn333::new(Back, Half),
    Turn333::new(Left, Clockwise),
    Turn333::new(Left, CounterClockwise),
    Turn333::new(Left, Half),
    Turn333::new(Right, Clockwise),
    Turn333::new(Right, CounterClockwise),
    Turn333::new(Right, Half),
];

pub const FB_EO_STATE_CHANGE_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Front, Clockwise),
    Turn333::new(Front, CounterClockwise),
    Turn333::new(Back, Clockwise),
    Turn333::new(Back, CounterClockwise),
];

pub const FB_EO_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Up, Half),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
    Turn333::new(Down, Half),
    Turn333::new(Front, Half),
    Turn333::new(Back, Half),
    Turn333::new(Left, Clockwise),
    Turn333::new(Left, CounterClockwise),
    Turn333::new(Left, Half),
    Turn333::new(Right, Clockwise),
    Turn333::new(Right, CounterClockwise),
    Turn333::new(Right, Half),
];

pub const RL_EO_STATE_CHANGE_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Right, Clockwise),
    Turn333::new(Left, CounterClockwise),
    Turn333::new(Right, Clockwise),
    Turn333::new(Left, CounterClockwise),
];

pub const RL_EO_MOVES: &[CubeOuterTurn] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Up, Half),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
    Turn333::new(Down, Half),
    Turn333::new(Front, Clockwise),
    Turn333::new(Front, CounterClockwise),
    Turn333::new(Front, Half),
    Turn333::new(Back, Clockwise),
    Turn333::new(Back, CounterClockwise),
    Turn333::new(Back, Half),
    Turn333::new(Left, Half),
    Turn333::new(Right, Half),
];

pub const EO_UD_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: UD_EO_STATE_CHANGE_MOVES,
    aux_moves: UD_EO_MOVES,
    transitions: &eo_transitions(Up),
};

pub const EO_FB_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: FB_EO_STATE_CHANGE_MOVES,
    aux_moves: FB_EO_MOVES,
    transitions: &eo_transitions(Front),
};

pub const EO_LR_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: RL_EO_STATE_CHANGE_MOVES,
    aux_moves: RL_EO_MOVES,
    transitions: &eo_transitions(Left),
};

pub const EO_UD_PRE_TRANS: [Transformation333; 1] = [Transformation333::new(CubeAxis::X, Clockwise)];
pub const EO_LR_PRE_TRANS: [Transformation333; 1] = [Transformation333::new(CubeAxis::Y, Clockwise)];
const BAD_EDGE_HEURISTIC: [u8; 7] = [0, 2, 1, 2, 2, 3, 3];

pub type EOPruningTable = PruningTable<2048, EOCoordFB>;

pub struct EOStepTable<'a> {
    move_set: &'a MoveSet333,
    pre_trans: Vec<Transformation333>,
    table: &'a EOPruningTable,
    name: &'a str,
}

pub fn from_step_config(table: &EOPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "eoud" | "ud" => Ok(CubeAxis::UD),
            "eofb" | "fb" => Ok(CubeAxis::FB),
            "eolr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid EO substep {x}"))
        }).collect();
        eo(table, axis?)
    } else {
        eo_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(5),
        config.absolute_min,
        config.absolute_max,
        config.niss.unwrap_or(NissSwitchType::Always),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn eo_any(table: &EOPruningTable) -> Step333 {
    eo(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn eo(table: &EOPruningTable, eo_axis: Vec<CubeAxis>) -> Step333 {
    let step_variants = eo_axis
        .into_iter()
        .map(move |x| {
            let x: Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333>> = match x {
                CubeAxis::UD => Box::new(EOStepTable::new_ud(&table)),
                CubeAxis::FB => Box::new(EOStepTable::new_fb(&table)),
                CubeAxis::LR => Box::new(EOStepTable::new_lr(&table)),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::EO, true)
}

impl<'a> EOStepTable<'a> {
    fn new_ud(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![Transformation333::new(CubeAxis::X, Clockwise)],
            table,
            name: "ud",
        }
    }

    fn new_lr(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![Transformation333::new(CubeAxis::Y, Clockwise)],
            table,
            name: "lr",
        }
    }

    fn new_fb(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![],
            table,
            name: "fb",
        }
    }
}

impl PreStepCheck<Turn333, Transformation333, Cube333> for EOStepTable<'_> {
    fn is_cube_ready(&self, _: &Cube333) -> bool {
        true
    }
}

impl PostStepCheck<Turn333, Transformation333, Cube333> for EOStepTable<'_> {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm<Turn333>) -> bool {
        true
    }
}

impl StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> for EOStepTable<'_> {
    fn move_set(&self, _: &Cube333, _: u8) -> &MoveSet333 {
        self.move_set
    }

    fn pre_step_trans(&self) -> &Vec<Transformation333> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &Cube333, _: u8, can_niss: bool) -> u8 {
        if can_niss {
            let fb_edges = cube.count_bad_edges().1;
            BAD_EDGE_HEURISTIC[(fb_edges >> 1) as usize]
        } else {
            let coord = EOCoordFB::from(cube);
            self.table.get(coord).0
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

pub fn filter_eo_last_moves_pure(alg: &Algorithm<Turn333>) -> bool {
    filter_last_moves_pure(&alg.normal_moves) && filter_last_moves_pure(&alg.inverse_moves)
}

fn filter_last_moves_pure(vec: &Vec<Turn333>) -> bool {
    match vec.len() {
        0 => true,
        1 => vec[0].dir != CounterClockwise,
        n => {
            if vec[n - 1].dir == CounterClockwise {
                false
            } else {
                if vec[n - 1].face.opposite() == vec[n - 2].face {
                    vec[n - 2].dir != CounterClockwise
                } else {
                    true
                }
            }
        }
    }
}

pub trait EOCount {
    fn count_bad_edges(&self) -> (u8, u8, u8);
}

impl EOCount for Cube333 {
    fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

const BAD_EDGE_MASK_UD: u64 = 0x0808080808080808;
const BAD_EDGE_MASK_FB: u64 = 0x0404040404040404;
const BAD_EDGE_MASK_RL: u64 = 0x0202020202020202;

impl EOCount for EdgeCube333 {
    fn count_bad_edges(&self) -> (u8, u8, u8) {
        let edges = self.get_edges_raw();
        let ud = (edges[0] & BAD_EDGE_MASK_UD).count_ones()
            + (edges[1] & BAD_EDGE_MASK_UD).count_ones();
        let fb = (edges[0] & BAD_EDGE_MASK_FB).count_ones()
            + (edges[1] & BAD_EDGE_MASK_FB).count_ones();
        let rl = (edges[0] & BAD_EDGE_MASK_RL).count_ones()
            + (edges[1] & BAD_EDGE_MASK_RL).count_ones();
        (ud as u8, fb as u8, rl as u8)
    }
}

pub(crate) const fn eo_transitions(axis_face: CubeFace) -> [TransitionTable333; 18] {
    let mut transitions = [TransitionTable333::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable333::moves_to_mask([
        Turn333::new(axis_face, Clockwise),
        Turn333::new(axis_face, CounterClockwise),
        Turn333::new(axis_face.opposite(), Clockwise),
        Turn333::new(axis_face.opposite(), CounterClockwise),
    ]);
    while i < CubeFace::ALL.len() {
        transitions[Turn333::new(CubeFace::ALL[i], Clockwise).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        transitions[Turn333::new(CubeFace::ALL[i], Half).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        transitions[Turn333::new(CubeFace::ALL[i], CounterClockwise).to_id()] = TransitionTable333::new(
            TransitionTable333::DEFAULT_ALLOWED_AFTER[CubeFace::ALL[i] as usize],
            can_end_mask,
        );
        i += 1;
    }
    transitions[Turn333::new(axis_face, Half).to_id()] = TransitionTable333::new(
        TransitionTable333::DEFAULT_ALLOWED_AFTER[axis_face as usize],
        TransitionTable333::NONE,
    );
    transitions[Turn333::new(axis_face.opposite(), Half).to_id()] = TransitionTable333::new(
        TransitionTable333::DEFAULT_ALLOWED_AFTER[axis_face.opposite() as usize],
        TransitionTable333::NONE,
    );
    transitions
}
