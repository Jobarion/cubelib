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
use crate::solver_new::*;
use crate::solver_new::htr::HTRStepOptions;
use crate::solver_new::step::*;
use crate::solver_new::thread_util::ToWorker;
use crate::steps::coord::ZeroCoord;
use crate::steps::dr::coords::DRUDEOFBCoord;
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};
use crate::steps::step::{PostStepCheck, PreStepCheck};

pub type EOOptions = StepOptions<EOStepOptions, 5, 20>;
pub static EO_TABLE: LazyLock<EOPruningTable> = LazyLock::new(gen_eo);

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

impl Default for EOStepOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Into<NissSwitchType> for &EOStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct EOStep;

impl EOStep {
    pub fn new(opts: EOOptions) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = opts.eo_axis.iter()
            .map(|eo|match eo.clone() {
                CubeAxis::UD => (vec![Transformation333::X], eo.name()),
                CubeAxis::FB => (vec![], eo.name()),
                CubeAxis::LR => (vec![Transformation333::Y], eo.name()),
            })
            .map(|(trans, name)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(PruningTableStep::<2048, EOCoordFB, 0, ZeroCoord>  {
                    table: &EO_TABLE,
                    options: (&opts).into(),
                    pre_step_trans: trans,
                    name: name.to_string(),
                    kind: StepKind::EO,
                    post_step_check: vec![],
                    move_set: &EOFB_MOVESET,
                    _pc: Default::default(),
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
