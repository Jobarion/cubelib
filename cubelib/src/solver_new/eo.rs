use std::sync::LazyLock;
use std::time::Instant;

use itertools::Itertools;
use log::{debug, info};

use crate::cube::*;
use crate::defs::StepKind;
use crate::solver::lookup_table;
use crate::solver_new::*;
use crate::solver_new::group::StepGroup;
use crate::solver_new::step::*;
use crate::steps::coord::ZeroCoord;
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};

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

pub struct EOStep;
pub type EOBuilder = builder::EOBuilderInternal<false, false, false, false, false>;

impl EOStep {
    pub fn builder() -> EOBuilder {
        EOBuilder::default()
    }

    pub fn new(dfs: DFSParameters, axis: Vec<CubeAxis>) -> StepGroup {
        let variants = axis.into_iter()
            .map(|eo|match eo {
                CubeAxis::UD => (vec![Transformation333::X], eo.name()),
                CubeAxis::FB => (vec![], eo.name()),
                CubeAxis::LR => (vec![Transformation333::Y], eo.name()),
            })
            .map(|(trans, name)| StepGroup::single(Box::new(PruningTableStep::<2048, EOCoordFB, 0, ZeroCoord>  {
                table: &EO_TABLE,
                options: dfs.clone(),
                pre_step_trans: trans,
                name: name.to_string(),
                kind: StepKind::EO,
                post_step_check: vec![],
                move_set: &EOFB_MOVESET,
                _pc: Default::default(),
            })))
            .collect_vec();
        StepGroup::parallel(variants)
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

pub mod builder {
    use crate::cube::CubeAxis;
    use crate::defs::NissSwitchType;
    use crate::solver_new::eo::EOStep;
    use crate::solver_new::group::StepGroup;
    use crate::solver_new::step::DFSParameters;

    pub struct EOBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
        _d_eo_axis: Vec<CubeAxis>,
        _e_min_length: usize,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> EOBuilderInternal<A, B, C, D, E> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool, const _E: bool>(self) -> EOBuilderInternal<_A, _B, _C, _D, _E> {
            EOBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_niss: self._c_niss,
                _d_eo_axis: self._d_eo_axis,
                _e_min_length: self._e_min_length,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool, const E: bool> EOBuilderInternal<false, B, C, D, E> {
        pub fn max_length(mut self, max_length: usize) -> EOBuilderInternal<true, B, C, D, E> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool, const E: bool> EOBuilderInternal<A, false, C, D, E> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> EOBuilderInternal<A, true, C, D, E> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool, const E: bool> EOBuilderInternal<A, B, false, D, E> {
        pub fn niss(mut self, niss: NissSwitchType) -> EOBuilderInternal<A, B, true, D, E> {
            self._c_niss = niss;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const E: bool> EOBuilderInternal<A, B, C, false, E> {
        pub fn eo_axis(mut self, eo_axis: Vec<CubeAxis>) -> EOBuilderInternal<A, B, C, true, E> {
            self._d_eo_axis = eo_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> EOBuilderInternal<A, B, C, D, false> {
        pub fn min_length(mut self, min_length: usize) -> EOBuilderInternal<A, B, C, D, true> {
            self._e_min_length = min_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> EOBuilderInternal<A, B, C, D, E> {
        pub fn build(self) -> StepGroup {
            let dfs = DFSParameters {
                niss_type: self._c_niss,
                min_moves: self._e_min_length,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            EOStep::new(dfs, self._d_eo_axis)
        }
    }

    impl EOBuilderInternal<false, false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 5,
                _b_max_absolute_length: 5,
                _e_min_length: 0,
                _c_niss: NissSwitchType::Always,
                _d_eo_axis: vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z],
            }
        }
    }

    impl Default for EOBuilderInternal<false, false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }
}