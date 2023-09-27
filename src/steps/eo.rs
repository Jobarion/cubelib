use itertools::Itertools;

use crate::algs::Algorithm;
use crate::cli::StepConfig;
use crate::coords::eo::EOCoordFB;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation, FACES};
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search::{NissType, SearchOptions};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{IsReadyForStep, Step, StepVariant};

pub const UD_EO_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
];

pub const UD_EO_MOVES: &[Move] = &[
    Move(Up, Half),
    Move(Down, Half),
    Move(Front, Clockwise),
    Move(Front, CounterClockwise),
    Move(Front, Half),
    Move(Back, Clockwise),
    Move(Back, CounterClockwise),
    Move(Back, Half),
    Move(Left, Clockwise),
    Move(Left, CounterClockwise),
    Move(Left, Half),
    Move(Right, Clockwise),
    Move(Right, CounterClockwise),
    Move(Right, Half),
];

pub const FB_EO_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Front, Clockwise),
    Move(Front, CounterClockwise),
    Move(Back, Clockwise),
    Move(Back, CounterClockwise),
];

pub const FB_EO_MOVES: &[Move] = &[
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Up, Half),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
    Move(Down, Half),
    Move(Front, Half),
    Move(Back, Half),
    Move(Left, Clockwise),
    Move(Left, CounterClockwise),
    Move(Left, Half),
    Move(Right, Clockwise),
    Move(Right, CounterClockwise),
    Move(Right, Half),
];

pub const RL_EO_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Right, Clockwise),
    Move(Left, CounterClockwise),
    Move(Right, Clockwise),
    Move(Left, CounterClockwise),
];

pub const RL_EO_MOVES: &[Move] = &[
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Up, Half),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
    Move(Down, Half),
    Move(Front, Clockwise),
    Move(Front, CounterClockwise),
    Move(Front, Half),
    Move(Back, Clockwise),
    Move(Back, CounterClockwise),
    Move(Back, Half),
    Move(Left, Half),
    Move(Right, Half),
];

pub const EO_UD_MOVESET: MoveSet = MoveSet {
    st_moves: UD_EO_STATE_CHANGE_MOVES,
    aux_moves: UD_EO_MOVES,
    transitions: eo_transitions(Up),
};

pub const EO_FB_MOVESET: MoveSet = MoveSet {
    st_moves: FB_EO_STATE_CHANGE_MOVES,
    aux_moves: FB_EO_MOVES,
    transitions: eo_transitions(Front),
};

pub const EO_LR_MOVESET: MoveSet = MoveSet {
    st_moves: RL_EO_STATE_CHANGE_MOVES,
    aux_moves: RL_EO_MOVES,
    transitions: eo_transitions(Left),
};

pub const EO_UD_PRE_TRANS: [Transformation; 1] = [Transformation(Axis::X, Clockwise)];
pub const EO_LR_PRE_TRANS: [Transformation; 1] = [Transformation(Axis::Y, Clockwise)];


pub type EOPruningTable = PruningTable<2048, EOCoordFB>;

pub struct EOStepTable<'a> {
    move_set: &'a MoveSet,
    pre_trans: Vec<Transformation>,
    table: &'a EOPruningTable,
    name: &'a str,
}

pub fn from_step_config<'a, C: 'a>(table: &'a EOPruningTable, config: StepConfig) -> Result<(Step<'a, C>, SearchOptions), String>
    where
        EOCoordFB: for<'x> From<&'x C>,{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "eoud" | "ud" => Ok(Axis::UD),
            "eofb" | "fb" => Ok(Axis::FB),
            "eolr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid EO substep {x}"))
        }).collect();
        eo(table, axis?)
    } else {
        eo_any(table)
    };
    let search_opts = SearchOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(5),
        config.niss.unwrap_or(NissType::During),
        config.quality,
        config.solution_count

    );
    Ok((step, search_opts))
}

pub fn eo_any<'a, C: 'a>(table: &'a EOPruningTable) -> Step<'a, C>
where
    EOCoordFB: for<'x> From<&'x C>,
{
    eo(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn eo<'a, C: 'a>(
    table: &'a EOPruningTable,
    eo_axis: Vec<Axis>,
) -> Step<'a, C>
where
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = eo_axis
        .into_iter()
        .map(move |x| {
            let x: Box<dyn StepVariant<C> + 'a> = match x {
                Axis::UD => Box::new(EOStepTable::new_ud(&table)),
                Axis::FB => Box::new(EOStepTable::new_fb(&table)),
                Axis::LR => Box::new(EOStepTable::new_lr(&table)),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "eo")
}

impl<'a> EOStepTable<'a> {
    fn new_ud(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::X, Clockwise)],
            table,
            name: "eoud",
        }
    }

    fn new_lr(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![Transformation(Axis::Y, Clockwise)],
            table,
            name: "eolr",
        }
    }

    fn new_fb(table: &'a EOPruningTable) -> Self {
        EOStepTable {
            move_set: &EO_FB_MOVESET,
            pre_trans: vec![],
            table,
            name: "eofb",
        }
    }
}

impl<'a, C> IsReadyForStep<C> for EOStepTable<'a>
where
    EOCoordFB: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, _: &C) -> bool {
        true
    }
}

impl<'a, C> StepVariant<C> for EOStepTable<'a>
where
    EOCoordFB: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C) -> u8 {
        let coord = EOCoordFB::from(cube);
        self.table.get(coord).expect("Expected table to be filled")
    }

    fn name(&self) -> &str {
        self.name
    }
}

pub fn filter_eo_last_moves_pure(alg: &Algorithm) -> bool {
    filter_last_moves_pure(&alg.normal_moves) && filter_last_moves_pure(&alg.inverse_moves)
}

fn filter_last_moves_pure(vec: &Vec<Move>) -> bool {
    match vec.len() {
        0 => true,
        1 => vec[0].1 != CounterClockwise,
        n => {
            if vec[n - 1].1 == CounterClockwise {
                false
            } else {
                if vec[n - 1].0.opposite() == vec[n - 2].0 {
                    vec[n - 2].1 != CounterClockwise
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

impl EOCount for CubieCube {
    fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

impl EOCount for EdgeCubieCube {
    fn count_bad_edges(&self) -> (u8, u8, u8) {
        let edges = self.get_edges_raw();
        let ud = (edges[0] & Self::BAD_EDGE_MASK_UD).count_ones()
            + (edges[1] & Self::BAD_EDGE_MASK_UD).count_ones();
        let fb = (edges[0] & Self::BAD_EDGE_MASK_FB).count_ones()
            + (edges[1] & Self::BAD_EDGE_MASK_FB).count_ones();
        let rl = (edges[0] & Self::BAD_EDGE_MASK_RL).count_ones()
            + (edges[1] & Self::BAD_EDGE_MASK_RL).count_ones();
        (ud as u8, fb as u8, rl as u8)
    }
}

pub(crate) const fn eo_transitions(axis_face: Face) -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable::moves_to_mask([
        Move(axis_face, Clockwise),
        Move(axis_face, CounterClockwise),
        Move(axis_face.opposite(), Clockwise),
        Move(axis_face.opposite(), CounterClockwise),
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
    transitions[Move(axis_face, Half).to_id()] = TransitionTable::new(
        TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face as usize],
        TransitionTable::NONE,
    );
    transitions[Move(axis_face.opposite(), Half).to_id()] = TransitionTable::new(
        TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face.opposite() as usize],
        TransitionTable::NONE,
    );
    transitions
}
