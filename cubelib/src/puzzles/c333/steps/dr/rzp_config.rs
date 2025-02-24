use itertools::Itertools;

use crate::algs::Algorithm;
use crate::defs::*;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{dr, MoveSet333, Step333};
use crate::puzzles::cube::{CubeAxis, CubeFace};
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::Direction::*;
use crate::solver::moveset::TransitionTable333;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

const QT_MOVES: [Turn333; 12] = [
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
    Turn333::new(Front, Clockwise),
    Turn333::new(Front, CounterClockwise),
    Turn333::new(Back, Clockwise),
    Turn333::new(Back, CounterClockwise),
    Turn333::new(Left, Clockwise),
    Turn333::new(Left, CounterClockwise),
    Turn333::new(Right, Clockwise),
    Turn333::new(Right, CounterClockwise),
];

pub const RZP_EO_FB_STATE_CHANGE_MOVES: &[Turn333] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
    Turn333::new(Right, Clockwise),
    Turn333::new(Right, CounterClockwise),
    Turn333::new(Left, Clockwise),
    Turn333::new(Left, CounterClockwise),
];

pub const RZP_EO_FB_AUX_MOVES: &[Turn333] = &[
    Turn333::new(Up, Half),
    Turn333::new(Down, Half),
    Turn333::new(Right, Half),
    Turn333::new(Left, Half),
    Turn333::new(Front, Half),
    Turn333::new(Back, Half),
];

pub const RZP_EO_FB_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: RZP_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: RZP_EO_FB_AUX_MOVES,
    transitions: &rzp_transitions(Left),
};

pub const RZP_ANY: MoveSet333 = MoveSet333 {
    st_moves: &QT_MOVES,
    aux_moves: dr::dr_config::HTR_MOVES,
    transitions: &rzp_transitions_any(),
};

pub struct RZPStep<'a> {
    move_set: &'a MoveSet333,
    pre_trans: Vec<Transformation333>,
    name: &'a str,
    is_any: bool,
}

pub fn from_step_config<'a>(config: StepConfig) -> Result<(Step333<'a>, DefaultStepOptions), String> {
    // let step = rzp_any();
    let step = rzp(vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z]);

    if !config.params.is_empty() {
        return Err(format!("Unreognized parameters: {:?}", config.params.keys()))
    }

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(3),
        config.absolute_min,
        config.absolute_max,
        config.niss.unwrap_or(NissSwitchType::Never),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 10))
        }
    );
    Ok((step, search_opts))
}

pub fn rzp_any<'a>() -> Step333<'a> {
    Step::new(vec![
        Box::new(RZPStep::new_any()),
    ], StepKind::RZP, false)
}


pub fn rzp<'a>(eo_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = eo_axis
        .into_iter()
        .map(move |x| {
            let x: Box<dyn StepVariant> = match x {
                CubeAxis::UD => Box::new(RZPStep::new_ud()),
                CubeAxis::FB => Box::new(RZPStep::new_fb()),
                CubeAxis::LR => Box::new(RZPStep::new_lr()),
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
            is_any: true,
        }
    }

    fn new_ud() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation333::X],
            name: "ud",
            is_any: false,
        }
    }

    fn new_fb() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![],
            name: "fb",
            is_any: false,
        }
    }

    fn new_lr() -> Self {
        RZPStep {
            move_set: &RZP_EO_FB_MOVESET,
            pre_trans: vec![Transformation333::Y],
            name: "lr",
            is_any: false,
        }
    }
}

impl<'a> PreStepCheck for RZPStep<'a> {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        let (ud, fb, lr) = cube.count_bad_edges();
        if self.is_any {
            ud == 0 || fb == 0 || lr == 0
        } else {
            fb == 0
        }
    }
}

impl<'a> PostStepCheck for RZPStep<'a> {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm) -> bool {
        true
    }
}

impl<'a> StepVariant for RZPStep<'a> {
    fn move_set(&self, _: &Cube333, _: u8) -> &'a MoveSet333 {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_trans
    }

    fn heuristic(&self, _: &Cube333, depth_left: u8, _: bool) -> u8 {
        depth_left //RZP is a special step without a real goal. Filtering by bad edge/corner count is done in subsequent DR steps
    }

    fn name(&self) -> &str {
        self.name
    }
}

const fn rzp_transitions(axis_face: CubeFace) -> [TransitionTable333; 18] {
    let mut transitions = [TransitionTable333::new(0, 0); 18];
    let mut i = 0;

    let can_end_mask = TransitionTable333::moves_to_mask([
        Turn333::U, Turn333::Ui,
        Turn333::D, Turn333::Di,
        Turn333::F, Turn333::Fi,
        Turn333::B, Turn333::Bi,
        Turn333::L, Turn333::Li,
        Turn333::R, Turn333::Ri]);

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

const fn rzp_transitions_any() -> [TransitionTable333; 18] {
    let mut transitions = [TransitionTable333::new(0, 0); 18];
    let mut i = 0;
    let can_end_mask = TransitionTable333::moves_to_mask(QT_MOVES);
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
    transitions
}
