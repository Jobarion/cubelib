use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;

use itertools::Itertools;
use log::{debug, info};

use crate::cube::*;
use crate::defs::StepKind;
use crate::solver::lookup_table;
use crate::solver_new::*;
use crate::solver_new::group::StepGroup;
use crate::solver_new::htr::HTR_TABLES;
use crate::solver_new::step::*;
use crate::solver_new::thread_util::ToWorker;
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable};
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::htr::subsets::{DRSubsetFilter, Subset};
use crate::steps::step::PostStepCheck;

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


pub type DRBuilder = builder::DRBuilderInternal<false, false, false, false, false, false, false>;
pub const DRUD_EOFB_MOVESET: MoveSet = MoveSet::new(DRUD_EOFB_ST_MOVES, DRUD_EOFB_AUX_MOVES);

pub struct DRStep;

impl DRStep {
    pub fn builder() -> DRBuilder {
        DRBuilder::default()
    }

    pub fn new(dfs_parameters: DFSParameters, axis: HashMap<CubeAxis, Vec<CubeAxis>>, subsets: Vec<Subset>) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = axis.into_iter()
            .flat_map(move |(dr, eo)|eo.into_iter().map(move |eo|(eo, dr.clone())))
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
                if !subsets.is_empty() {
                    post_checks.push(Box::new(DRSubsetFilter::new_subset(&HTR_TABLES.1, &subsets)));
                }
                (x, post_checks)
            })
            .map(|((trans, name), psc)|{
                let b: Box<dyn ToWorker + Send + 'static> = Box::new(PruningTableStep::<DRUDEOFB_SIZE, DRUDEOFBCoord, 2048, EOCoordFB> {
                    table: &DR_TABLE,
                    options: dfs_parameters.clone(),
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
            StepGroup::parallel(variants)
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

mod builder {
    use std::collections::HashMap;

    use crate::cube::CubeAxis;
    use crate::defs::NissSwitchType;
    use crate::solver_new::dr::DRStep;
    use crate::solver_new::step::DFSParameters;
    use crate::solver_new::thread_util::ToWorker;
    use crate::steps::util::Subset;

    pub struct DRBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const F: bool, const G: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
        _d_dr_eo_axis: HashMap<CubeAxis, Vec<CubeAxis>>,
        _e_subsets: Vec<Subset>,
        _f_triggers: Vec<String>,
        _g_rzp_options: Option<()>,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, C, D, E, F, G> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool, const _E: bool, const _F: bool, const _G: bool>(self) -> DRBuilderInternal<_A, _B, _C, _D, _E, _F, _G> {
            DRBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_niss: self._c_niss,
                _d_dr_eo_axis: self._d_dr_eo_axis,
                _e_subsets: self._e_subsets,
                _f_triggers: self._f_triggers,
                _g_rzp_options: self._g_rzp_options,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool, const E: bool, const F: bool, const G: bool> DRBuilderInternal<false, B, C, D, E, F, G> {
        pub fn max_length(mut self, max_length: usize) -> DRBuilderInternal<true, B, C, D, E, F, G> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool, const E: bool, const F: bool, const G: bool> DRBuilderInternal<A, false, C, D, E, F, G> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> DRBuilderInternal<A, true, C, D, E, F, G> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool, const E: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, false, D, E, F, G> {
        pub fn niss(mut self, niss: NissSwitchType) -> DRBuilderInternal<A, B, true, D, E, F, G> {
            self._c_niss = niss;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const E: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, C, false, E, F, G> {
        pub fn axis(mut self, dr_eo_axis: HashMap<CubeAxis, Vec<CubeAxis>>) -> DRBuilderInternal<A, B, C, true, E, F, G> {
            self._d_dr_eo_axis = dr_eo_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, C, D, false, F, G> {
        pub fn subsets(mut self, subsets: Vec<Subset>) -> DRBuilderInternal<A, B, C, D, true, F, G> {
            self._e_subsets = subsets;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const E: bool, const D: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, C, D, E, F, G> {
        pub fn add_subset(mut self, subset: Subset) -> DRBuilderInternal<A, B, C, D, true, F, G> {
            self._e_subsets.push(subset);
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const E: bool, const D: bool, const F: bool, const G: bool> DRBuilderInternal<A, B, C, D, E, F, G> {
        pub fn add_subsets<Y: AsRef<Subset>, X: IntoIterator<Item = Y>>(mut self, subsets: X) -> DRBuilderInternal<A, B, C, D, true, F, G> {
            for subset in subsets.into_iter() {
                self._e_subsets.push(subset.as_ref().clone());
            }
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const G: bool> DRBuilderInternal<A, B, C, D, E, false, G> {
        pub fn triggers(mut self, triggers: Vec<String>) -> DRBuilderInternal<A, B, C, D, E, true, G> {
            self._f_triggers = triggers;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const F: bool> DRBuilderInternal<A, B, C, D, E, F, false> {
        pub fn rzp(mut self, rzp: ()) -> DRBuilderInternal<A, B, C, D, E, F, true> {
            self._g_rzp_options = Some(rzp);
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> DRBuilderInternal<A, B, C, D, E, true, true> {
        pub fn build(self) -> Box<dyn ToWorker + Send + 'static> {
            unimplemented!("DRs with triggers are not supported yet")
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> DRBuilderInternal<A, B, C, D, E, false, false> {
        pub fn build(self) -> Box<dyn ToWorker + Send + 'static> {
            let dfs = DFSParameters {
                niss_type: self._c_niss,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            DRStep::new(dfs, self._d_dr_eo_axis, self._e_subsets)
        }
    }

    impl DRBuilderInternal<false, false, false, false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 11,
                _b_max_absolute_length: 13,
                _c_niss: NissSwitchType::Before,
                _d_dr_eo_axis: HashMap::from([(CubeAxis::X, vec![CubeAxis::Y, CubeAxis::Z]), (CubeAxis::Y, vec![CubeAxis::X, CubeAxis::Z]), (CubeAxis::Z, vec![CubeAxis::X, CubeAxis::Y])]),
                _e_subsets: vec![],
                _f_triggers: vec![],
                _g_rzp_options: None,
            }
        }
    }

    impl Default for DRBuilderInternal<false, false, false, false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    // impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> DRBuilderInternal<A, B, C, D, E> {
    //     fn prototype(mut self) -> DRBuilderInternal<A, B, C, D, E> {
    //         self.convert()
    //     }
    // }

}