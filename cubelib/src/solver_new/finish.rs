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
use crate::solver_new::thread_util::ToWorker;
use crate::steps::finish::coords::{FR_FINISH_SIZE, FRUDFinishCoord, HTR_FINISH_SIZE, HTR_LEAVE_SLICE_FINISH_SIZE, HTRFinishCoord, HTRLeaveSliceFinishCoord};
use crate::steps::finish::finish_config::{FRFinishPruningTable, FRUD_FINISH_MOVESET, HTR_FINISH_MOVESET, HTRFinishPruningTable, HTRLeaveSliceFinishPruningTable};
use crate::steps::fr::coords::{FRUD_NO_SLICE_SIZE, FRUD_WITH_SLICE_SIZE, FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};

pub static FR_FINISH_TABLE: LazyLock<FRFinishPruningTable> = LazyLock::new(||gen_fr_finish());
pub static HTR_FINISH_TABLE: LazyLock<HTRFinishPruningTable> = LazyLock::new(||gen_htr_finish());
pub static HTR_LEAVE_SLICE_FINISH_TABLE: LazyLock<HTRLeaveSliceFinishPruningTable> = LazyLock::new(||gen_htr_ls_finish());

const FINISH_FRUD_ST_MOVES: &[Turn333] = &[
    Turn333::F2, Turn333::B2,
    Turn333::L2, Turn333::R2,
];

const FINISH_HTR_ST_MOVES: &[Turn333] = &[
    Turn333::U2, Turn333::D2,
    Turn333::F2, Turn333::B2,
    Turn333::L2, Turn333::R2,
];

const FINISH_AUX_MOVES: &[Turn333] = &[];

pub const FINISH_FRUD_MOVESET: MoveSet = MoveSet::new(FINISH_FRUD_ST_MOVES, FINISH_AUX_MOVES);
pub const FINISH_HTR_MOVESET: MoveSet = MoveSet::new(FINISH_HTR_ST_MOVES, FINISH_AUX_MOVES);

pub struct FRFinishStep;
pub type FRFinishBuilder = builder::FRFinish<false, false, false, false>;

impl FRFinishStep {
    pub fn builder() -> FRFinishBuilder {
        FRFinishBuilder::default()
    }
}

impl FRFinishStep {
    pub fn new(dfs: DFSParameters, fr_axis: Vec<CubeAxis>, leave_slice: bool) -> Box<dyn ToWorker + Send + 'static> {
        let mut variants = fr_axis.into_iter()
            .map(|dr|match dr {
                CubeAxis::UD => (vec![], dr.name()),
                CubeAxis::FB => (vec![Transformation333::X], dr.name()),
                CubeAxis::LR => (vec![Transformation333::Z], dr.name()),
            })
            .map(|(trans, name)|{
                let b: Box<dyn ToWorker + Send + 'static> = if leave_slice {
                    Box::new(PruningTableStep::<FR_FINISH_SIZE, FRUDFinishCoord, FRUD_NO_SLICE_SIZE, FRUDNoSliceCoord>  {
                        table: &FR_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FINLS,
                        post_step_check: vec![],
                        move_set: &FINISH_FRUD_MOVESET,
                        _pc: Default::default(),
                    })
                } else {
                    Box::new(PruningTableStep::<FR_FINISH_SIZE, FRUDFinishCoord, FRUD_WITH_SLICE_SIZE, FRUDWithSliceCoord>  {
                        table: &FR_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FIN,
                        post_step_check: vec![],
                        move_set: &FINISH_FRUD_MOVESET,
                        _pc: Default::default(),
                    })
                };
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

pub struct HTRFinishStep;
pub type HTRFinishBuilder = builder::HTRFinish<false, false, false>;

impl HTRFinishStep {
    pub fn builder() -> HTRFinishBuilder {
        HTRFinishBuilder::default()
    }
}

impl HTRFinishStep {
    pub fn new(dfs: DFSParameters, leave_slice: bool) -> Box<dyn ToWorker + Send + 'static> {
        let b: Box<dyn ToWorker + Send + 'static> = if leave_slice {
            let variants = [CubeAxis::UD, CubeAxis::LR, CubeAxis::FB].into_iter()
                .map(|slice|match slice {
                    CubeAxis::UD => (vec![], slice.name()),
                    CubeAxis::FB => (vec![Transformation333::X], slice.name()),
                    CubeAxis::LR => (vec![Transformation333::Z], slice.name()),
                })
                .map(|(trans, name)|{
                    let b: Box<dyn ToWorker + Send + 'static> = Box::new(PruningTableStep::<HTR_LEAVE_SLICE_FINISH_SIZE, HTRLeaveSliceFinishCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                        table: &HTR_LEAVE_SLICE_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FINLS,
                        post_step_check: vec![],
                        move_set: &FINISH_HTR_MOVESET,
                        _pc: Default::default(),
                    });
                    b
                })
                .collect();
            StepGroup::parallel(variants)
        } else {
            Box::new(PruningTableStep::<HTR_FINISH_SIZE, HTRFinishCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                table: &HTR_FINISH_TABLE,
                options: dfs.clone(),
                pre_step_trans: vec![],
                name: "".to_string(),
                kind: StepKind::FIN,
                post_step_check: vec![],
                move_set: &FINISH_HTR_MOVESET,
                _pc: Default::default(),
            })
        };
        b
    }
}

fn gen_fr_finish() -> FRFinishPruningTable {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let fr_table = lookup_table::generate(&FRUD_FINISH_MOVESET,
                                          &|c: &Cube333| FRUDFinishCoord::from(c),
                                          &|| FRFinishPruningTable::new(false),
                                          &|table, coord|table.get(coord),
                                          &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    fr_table
}

fn gen_htr_finish() -> HTRFinishPruningTable {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let htr_fin_table = lookup_table::generate(&HTR_FINISH_MOVESET,
                                               &|c: &Cube333| HTRFinishCoord::from(c),
                                               &|| HTRFinishPruningTable::new(false),
                                               &|table, coord|table.get(coord),
                                               &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    htr_fin_table
}

fn gen_htr_ls_finish() -> HTRLeaveSliceFinishPruningTable {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let htr_fin_table = lookup_table::generate(&HTR_FINISH_MOVESET,
                                               &|c: &Cube333| HTRLeaveSliceFinishCoord::from(c),
                                               &|| HTRLeaveSliceFinishPruningTable::new(false),
                                               &|table, coord|table.get(coord),
                                               &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    htr_fin_table
}

pub mod builder {
    use crate::cube::CubeAxis;
    use crate::defs::NissSwitchType;
    use crate::solver_new::finish::{FRFinishStep, HTRFinishStep};
    use crate::solver_new::step::DFSParameters;
    use crate::solver_new::thread_util::ToWorker;

    pub struct FRFinish<const A: bool, const B: bool, const C: bool, const D: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_fr_axis: Vec<CubeAxis>,
        _d_leave_slice: bool,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> FRFinish<A, B, C, D> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool>(self) -> FRFinish<_A, _B, _C, _D> {
            FRFinish {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_fr_axis: self._c_fr_axis,
                _d_leave_slice: self._d_leave_slice,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool> FRFinish<false, B, C, D> {
        pub fn max_length(mut self, max_length: usize) -> FRFinish<true, B, C, D> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool> FRFinish<A, false, C, D> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> FRFinish<A, true, C, D> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool> FRFinish<A, B, false, D> {
        pub fn fr_axis(mut self, fr_axis: Vec<CubeAxis>) -> FRFinish<A, B, true, D> {
            self._c_fr_axis = fr_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool> FRFinish<A, B, false, D> {
        pub fn leave_slice(mut self) -> FRFinish<A, B, true, D> {
            self._d_leave_slice = true;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> FRFinish<A, B, C, D> {
        pub fn build(self) -> Box<dyn ToWorker + Send + 'static> {
            let dfs = DFSParameters {
                niss_type: NissSwitchType::Never,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            FRFinishStep::new(dfs, self._c_fr_axis, self._d_leave_slice)
        }
    }

    impl FRFinish<false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 8,
                _b_max_absolute_length: 24,
                _c_fr_axis: vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z],
                _d_leave_slice: false,
            }
        }
    }

    impl Default for FRFinish<false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    pub struct HTRFinish<const A: bool, const B: bool, const C: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_leave_slice: bool,
    }

    impl <const A: bool, const B: bool, const C: bool> HTRFinish<A, B, C> {
        fn convert<const _A: bool, const _B: bool, const _C: bool>(self) -> HTRFinish<_A, _B, _C> {
            HTRFinish {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_leave_slice: self._c_leave_slice,
            }
        }
    }

    impl <const B: bool, const C: bool> HTRFinish<false, B, C> {
        pub fn max_length(mut self, max_length: usize) -> HTRFinish<true, B, C> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool> HTRFinish<A, false, C> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> HTRFinish<A, true, C> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool> HTRFinish<A, B, false> {
        pub fn leave_slice(mut self) -> HTRFinish<A, B, true> {
            self._c_leave_slice = true;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool> HTRFinish<A, B, C> {
        pub fn build(self) -> Box<dyn ToWorker + Send + 'static> {
            let dfs = DFSParameters {
                niss_type: NissSwitchType::Never,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            HTRFinishStep::new(dfs, self._c_leave_slice)
        }
    }

    impl HTRFinish<false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 12,
                _b_max_absolute_length: 24,
                _c_leave_slice: false,
            }
        }
    }

    impl Default for HTRFinish<false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }
}
