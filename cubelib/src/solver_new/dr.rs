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
use crate::solver_new::group::{Parallel, Sequential};
use crate::solver_new::step::{DFSParameters, MoveSet, Receiver, Sender, Step, StepOptions};
use crate::solver_new::thread_util::{Run, ToWorker, Worker};
use crate::solver_new::util_steps::{Filter, StepFilter};
use crate::steps::dr::coords::DRUDEOFBCoord;
use crate::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable, HTR_DR_UD_MOVESET};
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::htr::coords::HTRDRUDCoord;
use crate::steps::htr::htr_config::{HTRPruningTable, HTRSubsetTable};
use crate::steps::htr::subsets::{DRSubsetFilter, Subset};
use crate::steps::step::{PostStepCheck, PreStepCheck};
use crate::steps::util::expand_subset_name;

pub type DROptions = StepOptions<DRStepOptions, 11, 13>;
static DR_TABLE: LazyLock<DRPruningTable> = LazyLock::new(gen_dr);
static DR_SUBSET_TABLE: LazyLock<HTRSubsetTable> = LazyLock::new(||gen_htr_with_subsets().1);

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

pub const DRUF_EOFB_MOVESET: MoveSet = MoveSet::new(DRUD_EOFB_ST_MOVES, DRUD_EOFB_AUX_MOVES);

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

impl Into<NissSwitchType> for &DRStepOptions {
    fn into(self) -> NissSwitchType {
        self.niss
    }
}

pub struct DRStep {
    pub table: &'static DRPruningTable,
    pub options: DROptions,
    pub pre_step_trans: Vec<Transformation333>,
    pub name: String
}

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
            .map(|(trans, name)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(Self {
                    table: &DR_TABLE,
                    options: opts.clone(),
                    pre_step_trans: trans,
                    name,
                });
                b
            })
            .collect_vec();
        let worker = if variants.len() == 1 {
            variants.pop().unwrap()
        } else {
            Box::new(Parallel::new(variants))
        };
        if !opts.subsets.is_empty() {
            Box::new(Sequential::new(vec![worker, Box::new(DRSubsetFilter::new_subset(&DR_SUBSET_TABLE, &opts.subsets))]))
        } else {
            worker
        }
    }
}

impl StepFilter for DRSubsetFilter<'static> {
    fn filter(&self, sol: &Solution, cube: &Cube333) -> bool {
        let mut cube = cube.clone();
        cube.apply_solution(sol);
        self.matches_subset(&cube)
    }
}

impl ToWorker for DRSubsetFilter<'static> {
    fn to_worker_box(self: Box<Self>, cube_state: Cube333, rc: Receiver<Solution>, tx: Sender<Solution>) -> Box<dyn Worker<()> + Send>
    where
        Self: Send + 'static
    {
        self.create_worker(cube_state, rc, tx)
    }
}

impl PreStepCheck for DRStep {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        EOCoordFB::from(cube).0 == 0
    }
}

impl PostStepCheck for DRStep {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm) -> bool {
        true
    }
}

impl <'a> Step for DRStep {
    fn get_dfs_parameters(&self) -> DFSParameters {
        (&self.options).into()
    }

    fn get_moveset(&self, _: &Cube333, _: usize) -> &'_ MoveSet {
        &DRUF_EOFB_MOVESET
    }

    fn heuristic(&self, state: &Cube333, can_niss_switch: bool) -> usize {
        if can_niss_switch {
            1
        } else {
            self.table.get(DRUDEOFBCoord::from(state)) as usize
        }
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_step_trans
    }

    fn get_name(&self) -> (StepKind, String) {
        (StepKind::DR, self.name.clone())
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

fn gen_htr_with_subsets() -> (HTRPruningTable, HTRSubsetTable) {
    info!("Generating DR pruning table...");
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