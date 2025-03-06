use std::sync::LazyLock;
use std::time::Instant;
use itertools::Itertools;
use log::{debug, info};
use typed_builder::TypedBuilder;
use crate::algs::Algorithm;
use crate::cube::*;
use crate::defs::{NissSwitchType, StepKind};
use crate::solver::lookup_table;
use crate::solver_new::group::Parallel;
use crate::solver_new::step::{DFSParameters, MoveSet, Step, StepOptions};
use crate::solver_new::thread_util::ToWorker;
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};
use crate::steps::step::{PostStepCheck, PreStepCheck};

pub type EOOptions = StepOptions<EOStepOptions, 5, 20>;
static EO_TABLE: LazyLock<EOPruningTable> = LazyLock::new(gen_eo);

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

#[derive(Clone, TypedBuilder)]
pub struct EOStepOptions {
    #[builder(default=NissSwitchType::Never)]
    pub niss: NissSwitchType,
    #[builder(default=vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z])]
    pub eo_axis: Vec<turn::CubeAxis>,
}

impl Into<NissSwitchType> for &EOStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct EOStep {
    table: &'static EOPruningTable,
    options: EOOptions,
    pre_step_trans: Vec<Transformation333>,
    name: &'static str,
}

impl EOStep {
    pub fn new(opts: EOOptions) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = opts.eo_axis.iter()
            .map(|eo|match eo.clone() {
                CubeAxis::UD => (vec![Transformation333::X], eo.name()),
                CubeAxis::FB => (vec![], eo.name()),
                CubeAxis::LR => (vec![Transformation333::Y], eo.name()),
            })
            .map(|(trans, name)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(Self {
                    table: &EO_TABLE,
                    options: opts.clone(),
                    pre_step_trans: trans,
                    name,
                });
                b
            })
            .collect_vec();
        if variants.len() == 1 {
            variants.pop().unwrap()
        } else {
            Box::new(Parallel::new(variants))
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
        (StepKind::EO, self.name.to_string())
    }
}

fn gen_eo() -> EOPruningTable {
    info!("Generating EO pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&EO_FB_MOVESET,
                                       &|c: &crate::cube::Cube333| EOCoordFB::from(c),
                                       &|| EOPruningTable::new(false),
                                       &|table, coord|table.get(coord),
                                       &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}
