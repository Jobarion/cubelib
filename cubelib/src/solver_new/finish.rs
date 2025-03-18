use std::sync::LazyLock;

use itertools::Itertools;
use log::debug;

use crate::cube::*;
use crate::defs::StepKind;
use crate::solver::lookup_table;
use crate::solver_new::*;
use crate::solver_new::group::StepGroup;
use crate::solver_new::step::*;
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
pub type FRFinishBuilder = builder::FRFinishBuilderInternal<false, false, false, false>;

impl FRFinishStep {
    pub fn builder() -> FRFinishBuilder {
        FRFinishBuilder::default()
    }
}

impl FRFinishStep {
    pub fn new(dfs: DFSParameters, fr_axis: Vec<CubeAxis>, leave_slice: bool) -> StepGroup {
        debug!("Step fin with options {dfs:?}");
        let variants = fr_axis.into_iter()
            .map(|dr|match dr {
                CubeAxis::UD => (vec![], dr.name()),
                CubeAxis::FB => (vec![Transformation333::X], dr.name()),
                CubeAxis::LR => (vec![Transformation333::Z], dr.name()),
            })
            .map(|(trans, name)|{
                if leave_slice {
                    StepGroup::single(Box::new(PruningTableStep::<FR_FINISH_SIZE, FRUDFinishCoord, FRUD_NO_SLICE_SIZE, FRUDNoSliceCoord>  {
                        table: &FR_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FINLS,
                        post_step_check: vec![],
                        move_set: &FINISH_FRUD_MOVESET,
                        _pc: Default::default(),
                    }))
                } else {
                    StepGroup::single(Box::new(PruningTableStep::<FR_FINISH_SIZE, FRUDFinishCoord, FRUD_WITH_SLICE_SIZE, FRUDWithSliceCoord>  {
                        table: &FR_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FIN,
                        post_step_check: vec![],
                        move_set: &FINISH_FRUD_MOVESET,
                        _pc: Default::default(),
                    }))
                }
            })
            .collect_vec();
        StepGroup::parallel(variants)
    }
}

pub struct HTRFinishStep;
pub type HTRFinishBuilder = builder::HTRFinishBuilderInternal<false, false, false>;

impl HTRFinishStep {
    pub fn builder() -> HTRFinishBuilder {
        HTRFinishBuilder::default()
    }
}

impl HTRFinishStep {
    pub fn new(dfs: DFSParameters, leave_slice: bool) -> StepGroup {
        debug!("Step fin with options {dfs:?}");
        if leave_slice {
            let variants = [CubeAxis::UD, CubeAxis::LR, CubeAxis::FB].into_iter()
                .map(|slice|match slice {
                    CubeAxis::UD => (vec![], slice.name()),
                    CubeAxis::FB => (vec![Transformation333::X], slice.name()),
                    CubeAxis::LR => (vec![Transformation333::Z], slice.name()),
                })
                .map(|(trans, name)|{
                    StepGroup::single(Box::new(PruningTableStep::<HTR_LEAVE_SLICE_FINISH_SIZE, HTRLeaveSliceFinishCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                        table: &HTR_LEAVE_SLICE_FINISH_TABLE,
                        options: dfs.clone(),
                        pre_step_trans: trans,
                        name: name.to_string(),
                        kind: StepKind::FINLS,
                        post_step_check: vec![],
                        move_set: &FINISH_HTR_MOVESET,
                        _pc: Default::default(),
                    }))
                })
                .collect();
            StepGroup::parallel(variants)
        } else {
            StepGroup::single(Box::new(PruningTableStep::<HTR_FINISH_SIZE, HTRFinishCoord, HTRDRUD_SIZE, HTRDRUDCoord>  {
                table: &HTR_FINISH_TABLE,
                options: dfs.clone(),
                pre_step_trans: vec![],
                name: "".to_string(),
                kind: StepKind::FIN,
                post_step_check: vec![],
                move_set: &FINISH_HTR_MOVESET,
                _pc: Default::default(),
            }))
        }
    }
}

fn gen_fr_finish() -> FRFinishPruningTable {
    FRFinishPruningTable::load_and_save("frfin", ||lookup_table::generate(&FRUD_FINISH_MOVESET,
                                          &|c: &Cube333| FRUDFinishCoord::from(c),
                                          &|| FRFinishPruningTable::new(false),
                                          &|table, coord|table.get(coord),
                                          &|table, coord, val|table.set(coord, val)))
}

fn gen_htr_finish() -> HTRFinishPruningTable {
    HTRFinishPruningTable::load_and_save("htrfin", ||lookup_table::generate(&HTR_FINISH_MOVESET,
                                               &|c: &Cube333| HTRFinishCoord::from(c),
                                               &|| HTRFinishPruningTable::new(false),
                                               &|table, coord|table.get(coord),
                                               &|table, coord, val|table.set(coord, val)))
}

fn gen_htr_ls_finish() -> HTRLeaveSliceFinishPruningTable {
    HTRLeaveSliceFinishPruningTable::load_and_save("htrfinls", ||lookup_table::generate(&HTR_FINISH_MOVESET,
                                               &|c: &Cube333| HTRLeaveSliceFinishCoord::from(c),
                                               &|| HTRLeaveSliceFinishPruningTable::new(false),
                                               &|table, coord|table.get(coord),
                                               &|table, coord, val|table.set(coord, val)))
}

pub mod builder {
    use crate::cube::CubeAxis;
    use crate::defs::{NissSwitchType, StepKind};
    use crate::solver_new::finish::{FRFinishStep, HTRFinishStep};
    use crate::solver_new::group::StepGroup;
    use crate::solver_new::step::DFSParameters;
    use crate::steps::step::StepConfig;

    pub struct FRFinishBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_fr_axis: Vec<CubeAxis>,
        _d_leave_slice: bool,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> FRFinishBuilderInternal<A, B, C, D> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool>(self) -> FRFinishBuilderInternal<_A, _B, _C, _D> {
            FRFinishBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_fr_axis: self._c_fr_axis,
                _d_leave_slice: self._d_leave_slice,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool> FRFinishBuilderInternal<false, B, C, D> {
        pub fn max_length(mut self, max_length: usize) -> FRFinishBuilderInternal<true, B, C, D> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool> FRFinishBuilderInternal<A, false, C, D> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> FRFinishBuilderInternal<A, true, C, D> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool> FRFinishBuilderInternal<A, B, false, D> {
        pub fn fr_axis(mut self, fr_axis: Vec<CubeAxis>) -> FRFinishBuilderInternal<A, B, true, D> {
            self._c_fr_axis = fr_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool> FRFinishBuilderInternal<A, B, false, D> {
        pub fn leave_slice(mut self) -> FRFinishBuilderInternal<A, B, true, D> {
            self._d_leave_slice = true;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> FRFinishBuilderInternal<A, B, C, D> {
        pub fn build(self) -> StepGroup {
            let dfs = DFSParameters {
                niss_type: NissSwitchType::Never,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            FRFinishStep::new(dfs, self._c_fr_axis, self._d_leave_slice)
        }
    }

    impl FRFinishBuilderInternal<false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 10,
                _b_max_absolute_length: 50,
                _c_fr_axis: vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z],
                _d_leave_slice: false,
            }
        }
    }

    impl Default for FRFinishBuilderInternal<false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TryFrom<StepConfig> for FRFinishBuilderInternal<false, false, false, false> {
        type Error = ();

        fn try_from(value: StepConfig) -> Result<Self, Self::Error> {
            if !value.params.is_empty() {
                return Err(())
            }
            if value.kind != StepKind::FIN && value.kind != StepKind::FINLS {
                return Err(())
            }
            let mut defaults = Self::default();
            if let Some(max) = value.max {
                defaults._a_max_length = max as usize;
            }
            if let Some(abs_max) = value.absolute_max {
                defaults._b_max_absolute_length = abs_max as usize;
            }
            if let Some(variants) = value.substeps {
                let axis: Result<Vec<CubeAxis>, Self::Error> = variants.into_iter()
                    .map(|variant| match variant.to_lowercase().as_str() {
                        "finishud" | "finud" | "ud" => Ok(CubeAxis::UD),
                        "finishfd" | "finfb" | "fb" => Ok(CubeAxis::FB),
                        "finishlr" | "finlr" | "lr" => Ok(CubeAxis::LR),
                        _ => Err(()),
                    })
                    .collect();
                defaults._c_fr_axis = axis?;
            }
            if value.kind == StepKind::FINLS {
                defaults._d_leave_slice = true;
            }
            Ok(defaults)
        }
    }

    pub struct HTRFinishBuilderInternal<const A: bool, const B: bool, const C: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_leave_slice: bool,
    }

    impl <const A: bool, const B: bool, const C: bool> HTRFinishBuilderInternal<A, B, C> {
        fn convert<const _A: bool, const _B: bool, const _C: bool>(self) -> HTRFinishBuilderInternal<_A, _B, _C> {
            HTRFinishBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_leave_slice: self._c_leave_slice,
            }
        }
    }

    impl <const B: bool, const C: bool> HTRFinishBuilderInternal<false, B, C> {
        pub fn max_length(mut self, max_length: usize) -> HTRFinishBuilderInternal<true, B, C> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool> HTRFinishBuilderInternal<A, false, C> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> HTRFinishBuilderInternal<A, true, C> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool> HTRFinishBuilderInternal<A, B, false> {
        pub fn leave_slice(mut self) -> HTRFinishBuilderInternal<A, B, true> {
            self._c_leave_slice = true;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool> HTRFinishBuilderInternal<A, B, C> {
        pub fn build(self) -> StepGroup {
            let dfs = DFSParameters {
                niss_type: NissSwitchType::Never,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            HTRFinishStep::new(dfs, self._c_leave_slice)
        }
    }

    impl HTRFinishBuilderInternal<false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 14,
                _b_max_absolute_length: 50,
                _c_leave_slice: false,
            }
        }
    }

    impl Default for HTRFinishBuilderInternal<false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TryFrom<StepConfig> for HTRFinishBuilderInternal<false, false, false> {
        type Error = ();

        fn try_from(value: StepConfig) -> Result<Self, Self::Error> {
            if !value.params.is_empty() {
                return Err(())
            }
            if value.kind != StepKind::FIN && value.kind != StepKind::FINLS {
                return Err(())
            }
            let mut defaults = Self::default();
            if let Some(max) = value.max {
                defaults._a_max_length = max as usize;
            }
            if let Some(abs_max) = value.absolute_max {
                defaults._b_max_absolute_length = abs_max as usize;
            }
            if value.kind == StepKind::FINLS {
                defaults._c_leave_slice = true;
            }
            Ok(defaults)
        }
    }
}
