use std::sync::Arc;
use typed_builder::TypedBuilder;
use crate::algs::Algorithm;
use crate::cube::{Cube333, CubeFace, Direction, Transformation333, Turn333};
use crate::defs::{NissSwitchType, StepKind};
use crate::solver::moveset::{Transition, TransitionTable333};
use crate::solver_new::step::{DFSParameters, MoveSet, Step, StepOptions};
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::eo::eo_config::EOPruningTable;
use crate::steps::step::{PostStepCheck, PreStepCheck};

pub type EOOptions = StepOptions<EOStepOptions, 5, 20>;


const EOFB_ST_MOVES: &[Turn333] = &[
    Turn333::F, Turn333::Fi,
    Turn333::B, Turn333::Bi,
];

const EOFB_AUX_MOVES: &[Turn333] = &[
    Turn333::U, Turn333::Ui, Turn333::U2,
    Turn333::D, Turn333::Di, Turn333::D2,
    Turn333::F2,
    Turn333::B2,
    Turn333::L, Turn333::Li, Turn333::L2,
    Turn333::R, Turn333::Ri, Turn333::R2,
];

pub const EOFB_MOVESET: MoveSet = MoveSet::new(EOFB_ST_MOVES, EOFB_AUX_MOVES);

#[derive(TypedBuilder)]
pub struct EOStepOptions {
    #[builder(default=NissSwitchType::Never)]
    pub niss: NissSwitchType
}

impl Into<NissSwitchType> for &EOStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct EOStep {
     table: Arc<EOPruningTable>,
     options: EOOptions,
     pre_step_trans: Vec<Transformation333>,
}

impl EOStep {
    pub fn new(opts: EOOptions, table: Arc<EOPruningTable>) -> Self {
        Self {
            table,
            options: opts,
            pre_step_trans: vec![],
        }
    }
}

impl PreStepCheck for EOStep {
    fn is_cube_ready(&self, _: &Cube333) -> bool {
        true
    }
}

impl PostStepCheck for EOStep {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm) -> bool {
        true
    }
}

impl <'a> Step for EOStep {
    fn get_dfs_parameters(&self) -> DFSParameters {
        (&self.options).into()
    }

    fn get_moveset(&self, _: &Cube333, _: usize) -> &'_ MoveSet {
        &EOFB_MOVESET
    }

    fn heuristic(&self, state: &Cube333, can_niss_switch: bool) -> usize {
        if can_niss_switch {
            1
        } else {
            self.table.get(EOCoordFB::from(state)) as usize
        }
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_step_trans
    }

    fn get_name(&self) -> (StepKind, String) {
        (StepKind::EO, "fb".to_string())
    }
}
