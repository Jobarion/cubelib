use itertools::Itertools;

use crate::algs::Algorithm;
use crate::cli::StepConfig;
use crate::coords::coord::Coord;
use crate::coords::eo::EOCoordFB;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation, FACES};
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search::{ALL_MOVES, NissType};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::{dr, eo, htr};
use crate::steps::eo::EOCount;
use crate::steps::step::{PreStepCheck, DefaultStepOptions, Step, StepVariant, PostStepCheck};

const QT_MOVES: [Move; 12] = [
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
    Move(Front, Clockwise),
    Move(Front, CounterClockwise),
    Move(Back, Clockwise),
    Move(Back, CounterClockwise),
    Move(Left, Clockwise),
    Move(Left, CounterClockwise),
    Move(Right, Clockwise),
    Move(Right, CounterClockwise),
];


//Exactly the same as DR
pub const RZP_EO_FB_MOVESET: MoveSet = MoveSet {
    st_moves: dr::DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: dr::DR_UD_EO_FB_MOVES,
    transitions: rzp_transitions(Left),
};

pub const RZP_ANY: MoveSet = MoveSet {
    st_moves: &QT_MOVES,
    aux_moves: htr::HTR_MOVES,
    transitions: rzp_transitions_any(),
};

pub struct RZPStep<'a> {
    move_set: &'a MoveSet,
    pre_trans: Vec<Transformation>,
    name: &'a str,
}

pub fn from_step_config<'a, C: 'a + EOCount>(config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String> {
    // let step = if let Some(substeps) = config.substeps {
    //     let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
    //         "rzpud" | "ud" => Ok(Axis::UD),
    //         "rzpfb" | "fb" => Ok(Axis::FB),
    //         "rzplr" | "lr" => Ok(Axis::LR),
    //         x => Err(format!("Invalid EO substep {x}"))
    //     }).collect();
    //     rzp(axis?)
    // } else {
    //     rzp_any()
    // };
    let step = rzp_any();
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(3),
        config.niss.unwrap_or(NissType::Before),
        config.quality,
        config.solution_count

    );
    Ok((step, search_opts))
}

pub fn rzp_any<'a, C: 'a + EOCount>() -> Step<'a, C> {
    Step::new(vec![
        Box::new(RZPStep::new_any()),
    ], "rzp")
    // Step::new(vec![
    //     Box::new(RZPStep::new_eoud()),
    //     Box::new(RZPStep::new_eofb()),
    //     Box::new(RZPStep::new_eolr()),
    // ], "rzp")
}


pub fn eo<'a, C: 'a + EOCount>(
    eo_axis: Vec<Axis>,
) -> Step<'a, C> {
    let step_variants = eo_axis
        .into_iter()
        .map(move |x| {
            let x: Box<dyn StepVariant<C> + 'a> = match x {
                Axis::UD => Box::new(RZPStep::new_ud()),
                Axis::FB => Box::new(RZPStep::new_fb()),
                Axis::LR => Box::new(RZPStep::new_lr()),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "rzp")
}

impl<'a> RZPStep<'a> {
    fn new_any() -> Self {
        RZPStep {
            move_set: &RZP_ANY,
            pre_trans: vec![],
            name: "rzp",
        }
    }

    fn new_ud() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation::X],
            name: "rzp-ud",
        }
    }

    fn new_fb() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![],
            name: "rzp-fb",
        }
    }

    fn new_lr() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation::Y],
            name: "rzp-lr",
        }
    }
}

impl<'a, C: EOCount> PreStepCheck<C> for RZPStep<'a>
{
    fn is_cube_ready(&self, cube: &C) -> bool {
        let (ud, fb, lr) = cube.count_bad_edges();
        ud == 0 || fb == 0 || lr == 0
    }
}

impl<'a, C> PostStepCheck<C> for RZPStep<'a> {
    fn is_solution_admissible(&self, cube: &C, alg: &Algorithm) -> bool {
        true
    }
}

impl<'a, CubeParam: EOCount> StepVariant<CubeParam> for RZPStep<'a>
{
    fn move_set(&self, cube: &CubeParam, depth_left: u8) -> &'a MoveSet {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, _: &CubeParam, depth_left: u8, _: bool) -> u8 {
        depth_left //RZP is a special step without a real goal. Filtering by bad edge/corner count is done in subsequent DR steps
    }

    fn name(&self) -> &str {
        self.name
    }

    fn is_half_turn_invariant(&self) -> bool {
        !self.move_set.st_moves
            .iter()
            .any(|m| m.1 == Half)
    }
}

const fn rzp_transitions(axis_face: Face) -> [TransitionTable; 18] {
    eo::eo_transitions(axis_face)
}

const fn rzp_transitions_any() -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable::moves_to_mask(QT_MOVES);
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
