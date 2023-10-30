use itertools::Itertools;

use crate::algs::Algorithm;
use crate::cube::{Axis, Face, FACES, Move, Transformation};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::defs::*;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::{dr, eo};
use crate::steps::eo::eo_config::EOCount;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

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
    st_moves: dr::dr_config::DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: dr::dr_config::DR_UD_EO_FB_MOVES,
    transitions: rzp_transitions(Left),
};

pub const RZP_ANY: MoveSet = MoveSet {
    st_moves: &QT_MOVES,
    aux_moves: dr::dr_config::HTR_MOVES,
    transitions: rzp_transitions_any(),
};

pub struct RZPStep<'a> {
    move_set: &'a MoveSet,
    pre_trans: Vec<Transformation>,
    name: &'a str,
}

pub fn from_step_config<'a, C: 'a + EOCount>(config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String> {
    let step = rzp_any();
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(3),
        config.niss.unwrap_or(NissSwitchType::Never),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 10))
        }

    );
    Ok((step, search_opts))
}

pub fn rzp_any<'a, C: 'a + EOCount>() -> Step<'a, C> {
    Step::new(vec![
        Box::new(RZPStep::new_any()),
    ], StepKind::RZP, false)
}


pub fn rzp<'a, C: 'a + EOCount>(
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
    Step::new(step_variants, StepKind::RZP, false)
}

impl<'a> RZPStep<'a> {
    fn new_any() -> Self {
        RZPStep {
            move_set: &RZP_ANY,
            pre_trans: vec![],
            name: "",
        }
    }

    fn new_ud() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation::X],
            name: "ud",
        }
    }

    fn new_fb() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![],
            name: "fb",
        }
    }

    fn new_lr() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation::Y],
            name: "lr",
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
    fn is_solution_admissible(&self, _: &C, _: &Algorithm) -> bool {
        true
    }
}

impl<'a, CubeParam: EOCount> StepVariant<CubeParam> for RZPStep<'a>
{
    fn move_set(&self, _: &CubeParam, _: u8) -> &'a MoveSet {
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
    eo::eo_config::eo_transitions(axis_face)
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
