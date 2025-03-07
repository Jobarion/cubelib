use std::sync::LazyLock;
use std::time::Instant;
use itertools::Itertools;
use log::{debug, info};
use typed_builder::TypedBuilder;
use crate::cube::*;
use crate::defs::{NissSwitchType, StepKind};
use crate::solver::lookup_table;
use crate::solver_new::group::Parallel;
use crate::solver_new::*;
use crate::solver_new::step::*;
use crate::solver_new::thread_util::ToWorker;
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};
use crate::steps::htr::htr_config::{HTRPruningTable, HTRSubsetTable};

pub type HTROptions = StepOptions<HTRStepOptions, 10, 18>;
pub static HTR_TABLES: LazyLock<(HTRPruningTable, HTRSubsetTable)> = LazyLock::new(||gen_htr_with_subsets());

const HTR_DRUD_ST_MOVES: &[Turn333] = &[
    Turn333::U, Turn333::Ui,
    Turn333::D, Turn333::Di,
];

const HTR_DRUD_AUX_MOVES: &[Turn333] = &[
    Turn333::U2, Turn333::D2,
    Turn333::F2, Turn333::B2,
    Turn333::L2, Turn333::R2,
];

pub const HTR_DRUD_MOVESET: MoveSet = MoveSet::new(HTR_DRUD_ST_MOVES, HTR_DRUD_AUX_MOVES);

#[derive(Clone, TypedBuilder)]
pub struct HTRStepOptions {
    #[builder(default=vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z])]
    pub dr_axis: Vec<turn::CubeAxis>,
    #[builder(default=NissSwitchType::Before)]
    pub niss: NissSwitchType,
}

impl Default for HTRStepOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Into<NissSwitchType> for &HTRStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct HTRStep;

impl HTRStep {
    pub fn new(opts: HTROptions) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = opts.dr_axis.iter()
            .map(|dr|match dr.clone() {
                CubeAxis::UD => (vec![], dr.name()),
                CubeAxis::FB => (vec![Transformation333::X], dr.name()),
                CubeAxis::LR => (vec![Transformation333::Z], dr.name()),
            })
            .map(|(trans, name)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(NissPruningTableStep::<HTRDRUD_SIZE, HTRDRUDCoord, DRUDEOFB_SIZE, DRUDEOFBCoord>  {
                    table: &HTR_TABLES.0,
                    options: (&opts).into(),
                    pre_step_trans: trans,
                    name: name.to_string(),
                    kind: StepKind::HTR,
                    post_step_check: vec![],
                    move_set: &HTR_DRUD_MOVESET,
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

fn gen_htr_with_subsets() -> (HTRPruningTable, HTRSubsetTable) {
    info!("Generating HTR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let mut htr_table = lookup_table::generate(&HTR_DR_UD_MOVESET,
                                               &|c: &Cube333| HTRDRUDCoord::from(c),
                                               &|| HTRPruningTable::new(),
                                               &|table, coord|table.get(coord).0,
                                               &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());

    info!("Generating HTR subset table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let subset_table = crate::steps::htr::subsets::gen_subset_tables(&mut htr_table);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    (htr_table, subset_table)
}