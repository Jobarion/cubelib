use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;

use itertools::Itertools;
use log::{debug, info, warn};
use tinyset::Set64;
use typed_builder::TypedBuilder;

use crate::algs::Algorithm;
use crate::cube::*;
use crate::defs::{NissSwitchType, StepKind};
use crate::solver::lookup_table;
use crate::solver::solution::{ApplySolution, Solution};
use crate::solver_new::*;
use crate::solver_new::group::{Parallel, Sequential};
use crate::solver_new::htr::{HTR_TABLES, HTRStepOptions};
use crate::solver_new::step::*;
use crate::solver_new::thread_util::{Run, ToWorker, Worker};
use crate::solver_new::util_steps::{Filter, StepFilter};
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable, HTR_DR_UD_MOVESET};
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::htr::coords::HTRDRUDCoord;
use crate::steps::htr::htr_config::{HTRPruningTable, HTRSubsetTable};
use crate::steps::htr::subsets::{DRSubsetFilter, Subset};
use crate::steps::step::{PostStepCheck, PreStepCheck};
use crate::steps::util::expand_subset_name;

pub type DROptions = StepOptions<DRStepOptions, 11, 13>;
pub static DR_TABLE: LazyLock<DRPruningTable> = LazyLock::new(gen_dr);

const DRUD_EOFB_ST_MOVES: &[Turn333] = &[
    Turn333::L, Turn333::Li,
    Turn333::R, Turn333::Ri,
];

const DRUD_EOFB_AUX_MOVES: &[Turn333] = &[
    Turn333::U, Turn333::Ui, Turn333::U2,
    Turn333::D, Turn333::Di, Turn333::D2,
    Turn333::F2,
    Turn333::B2,
    Turn333::L2,
    Turn333::R2,
];

pub const DRUD_EOFB_MOVESET: MoveSet = MoveSet::new(DRUD_EOFB_ST_MOVES, DRUD_EOFB_AUX_MOVES);

#[derive(Clone, TypedBuilder)]
pub struct DRStepOptions {
    #[builder(default=HashMap::from([(CubeAxis::X, vec![CubeAxis::Y, CubeAxis::Z]), (CubeAxis::Y, vec![CubeAxis::X, CubeAxis::Z]), (CubeAxis::Z, vec![CubeAxis::X, CubeAxis::Y])]))]
    pub dr_eo_axis: HashMap<turn::CubeAxis, Vec<turn::CubeAxis>>,
    #[builder(default=NissSwitchType::Before)]
    pub niss: NissSwitchType,
    // #[builder(default=vec![])]
    // pub triggers: Vec<Algorithm>,
    #[builder(default=vec![])]
    pub subsets: Vec<Subset>,
}

impl Default for DRStepOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Into<NissSwitchType> for &DRStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct DRStep;

impl DRStep {
    pub fn new(opts: DROptions) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = opts.dr_eo_axis.iter()
            .flat_map(|(dr, eo)|eo.iter().cloned().map(|eo|(eo, dr.clone())))
            .filter_map(|(eo,dr)|match (eo, dr) {
                (CubeAxis::UD, CubeAxis::FB) => Some((vec![Transformation333::X], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                (CubeAxis::UD, CubeAxis::LR) => Some((vec![Transformation333::X, Transformation333::Z], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                (CubeAxis::FB, CubeAxis::UD) => Some((vec![], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                (CubeAxis::FB, CubeAxis::LR) => Some((vec![Transformation333::Z], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                (CubeAxis::LR, CubeAxis::UD) => Some((vec![Transformation333::Y], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                (CubeAxis::LR, CubeAxis::FB) => Some((vec![Transformation333::Y, Transformation333::Z], format!("{}-eo{}", dr.name(), eo.name()).to_string())),
                _ => None,
            })
            .map(|x|{
                let mut post_checks: Vec<Box<dyn PostStepCheck + Send>> = vec![];
                if !opts.subsets.is_empty() {
                    post_checks.push(Box::new(DRSubsetFilter::new_subset(&HTR_TABLES.1, &opts.subsets)));
                }
                (x, post_checks)
            })
            .map(|((trans, name), psc)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(PruningTableStep::<DRUDEOFB_SIZE, DRUDEOFBCoord, 2048, EOCoordFB> {
                    table: &DR_TABLE,
                    options: (&opts).into(),
                    pre_step_trans: trans,
                    post_step_check: psc,
                    move_set: &DRUD_EOFB_MOVESET,
                    name,
                    kind: StepKind::DR,
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

fn gen_dr() -> DRPruningTable {
    info!("Generating DR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET,
                                       &|c: &crate::cube::Cube333| DRUDEOFBCoord::from(c),
                                       &|| DRPruningTable::new(false),
                                       &|table, coord|table.get(coord),
                                       &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}
