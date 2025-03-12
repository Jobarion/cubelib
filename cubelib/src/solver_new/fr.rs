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
use crate::steps::fr::coords::{FRUD_WITH_SLICE_SIZE, FRUD_NO_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::steps::fr::fr_config::{FR_UD_MOVESET, FRLeaveSlicePruningTable, FRPruningTable};
use crate::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};

pub static FR_TABLE: LazyLock<FRPruningTable> = LazyLock::new(||gen_fr());
pub static FR_LEAVE_SLICE_TABLE: LazyLock<FRLeaveSlicePruningTable> = LazyLock::new(||gen_frls());

const FRUD_ST_MOVES: &[Turn333] = &[
    Turn333::U2, Turn333::D2,
];

const FR_UD_AUX_MOVES: &[Turn333] = &[
    Turn333::F2, Turn333::B2,
    Turn333::L2, Turn333::R2,
];

pub const FRUD_MOVESET: MoveSet = MoveSet::new(FRUD_ST_MOVES, FR_UD_AUX_MOVES);

pub struct FRStep;
pub type FRBuilder = builder::FRBuilderInternal<false, false, false, false, false>;

impl FRStep {
    pub fn builder() -> FRBuilder {
        FRBuilder::default()
    }
}

impl FRStep {
    pub fn new(dfs: DFSParameters, fr_axis: Vec<CubeAxis>, leave_slice: bool) -> StepGroup {
        let variants = fr_axis.into_iter()
            .map(|dr|match dr {
                CubeAxis::UD => (vec![], dr.name()),
                CubeAxis::FB => (vec![Transformation333::X], dr.name()),
                CubeAxis::LR => (vec![Transformation333::Z], dr.name()),
            })
            .map(|(trans, name)|{
                if leave_slice {
                    StepGroup::single(Box::new(PruningTableStep::<FRUD_NO_SLICE_SIZE, FRUDNoSliceCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                        table: &FR_LEAVE_SLICE_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FR,
                        post_step_check: vec![],
                        move_set: &FRUD_MOVESET,
                        _pc: Default::default(),
                    }))
                } else {
                    StepGroup::single(Box::new(PruningTableStep::<FRUD_WITH_SLICE_SIZE, FRUDWithSliceCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                        table: &FR_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FR,
                        post_step_check: vec![],
                        move_set: &FRUD_MOVESET,
                        _pc: Default::default(),
                    }))
                }
            })
            .collect_vec();
        StepGroup::parallel(variants)
    }
}

fn gen_fr() -> FRPruningTable {
    info!("Generating FR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let fr_table = lookup_table::generate(&FR_UD_MOVESET,
                                              &|c: &Cube333| FRUDWithSliceCoord::from(c),
                                              &|| FRPruningTable::new(false),
                                              &|table, coord|table.get(coord),
                                              &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    fr_table
}

fn gen_frls() -> FRLeaveSlicePruningTable {
    info!("Generating FRLS pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let fr_table = lookup_table::generate(&FR_UD_MOVESET,
                                              &|c: &Cube333| FRUDNoSliceCoord::from(c),
                                              &|| FRLeaveSlicePruningTable::new(false),
                                              &|table, coord|table.get(coord),
                                              &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    fr_table
}


pub mod builder {
    use crate::cube::CubeAxis;
    use crate::defs::NissSwitchType;
    use crate::solver_new::fr::FRStep;
    use crate::solver_new::group::StepGroup;
    use crate::solver_new::step::DFSParameters;

    pub struct FRBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
        _d_fr_axis: Vec<CubeAxis>,
        _e_leave_slice: bool,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> FRBuilderInternal<A, B, C, D, E> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool, const _E: bool>(self) -> FRBuilderInternal<_A, _B, _C, _D, _E> {
            FRBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_niss: self._c_niss,
                _d_fr_axis: self._d_fr_axis,
                _e_leave_slice: self._e_leave_slice,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool, const E: bool> FRBuilderInternal<false, B, C, D, E> {
        pub fn max_length(mut self, max_length: usize) -> FRBuilderInternal<true, B, C, D, E> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool, const E: bool> FRBuilderInternal<A, false, C, D, E> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> FRBuilderInternal<A, true, C, D, E> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool, const E: bool> FRBuilderInternal<A, B, false, D, E> {
        pub fn niss(mut self, niss: NissSwitchType) -> FRBuilderInternal<A, B, true, D, E> {
            self._c_niss = niss;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const E: bool> FRBuilderInternal<A, B, C, false, E> {
        pub fn axis(mut self, eo_axis: Vec<CubeAxis>) -> FRBuilderInternal<A, B, C, true, E> {
            self._d_fr_axis = eo_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> FRBuilderInternal<A, B, C, D, false> {
        pub fn leave_slice(mut self) -> FRBuilderInternal<A, B, C, D, true> {
            self._e_leave_slice = true;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> FRBuilderInternal<A, B, C, D, E> {
        pub fn build(self) -> StepGroup {
            let dfs = DFSParameters {
                niss_type: self._c_niss,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            FRStep::new(dfs, self._d_fr_axis, self._e_leave_slice)
        }
    }

    impl FRBuilderInternal<false, false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 8,
                _b_max_absolute_length: 24,
                _c_niss: NissSwitchType::Before,
                _d_fr_axis: vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z],
                _e_leave_slice: false,
            }
        }
    }

    impl Default for FRBuilderInternal<false, false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }
}