use std::sync::LazyLock;

use itertools::Itertools;
use log::debug;

use crate::cube::*;
use crate::defs::StepVariant;
use crate::solver::lookup_table::{DepthEstimate, InMemoryIndexTable};
use crate::solver_new::dr_subset::table::generate_subset_table;
use crate::solver_new::group::StepGroup;
use crate::solver_new::htr::HTR_DRUD_MOVESET;
use crate::solver_new::step::*;
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};
use crate::steps::util::DR_SUBSETS;

pub static DR_SUBSET_TABLE: LazyLock<DRSubsetPruningTable> = LazyLock::new(||gen_htr_with_subsets());
pub type DRSubsetPruningTable = Box<dyn DepthEstimate<{HTRDRUD_SIZE}, HTRDRUDCoord>>;
pub type HTRSubsetTable = InMemoryIndexTable<{HTRDRUD_SIZE}, HTRDRUDCoord>;

pub struct DRSubsetStep;
pub type DRSubsetBuilder = builder::DRSubsetBuilderInternal<false, false, false, false>;

impl DRSubsetStep {
    pub fn builder() -> DRSubsetBuilder {
        DRSubsetBuilder::default()
    }
}

impl DRSubsetStep {
    pub fn new(dfs: DFSParameters, dr_axis: Vec<CubeAxis>) -> StepGroup {
        debug!("Step dr-subset step with options {dfs:?}");
        let variants = dr_axis.into_iter()
            .map(|dr|match dr {
                CubeAxis::UD => (vec![], dr),
                CubeAxis::FB => (vec![Transformation333::X], dr),
                CubeAxis::LR => (vec![Transformation333::Z], dr),
            })
            .map(|(trans, dr)|{
                StepGroup::single(Box::new(PruningTableStep::<HTRDRUD_SIZE, HTRDRUDCoord, DRUDEOFB_SIZE, DRUDEOFBCoord>  {
                    table: &DR_SUBSET_TABLE,
                    options: dfs.clone(),
                    pre_step_trans: trans,
                    pre_step_check: vec![],
                    variant: StepVariant::DR_4A1_4E(dr),
                    post_step_check: vec![],
                    move_set: &HTR_DRUD_MOVESET,
                    _pc: Default::default(),
                }))
            })
            .collect_vec();
        StepGroup::parallel(variants)
    }
}

fn gen_htr_with_subsets() -> DRSubsetPruningTable {
    let htr_table = InMemoryIndexTable::load_and_save("dr-4a1-4e", || generate_subset_table(&HTR_DR_UD_MOVESET,
                                                                                                &DR_SUBSETS[7],
                                                                                                &|c: &Cube333| HTRDRUDCoord::from(c),
                                                                                                &|| InMemoryIndexTable::new(false),
                                                                                                &|table, coord|table.get(coord),
                                                                                                &|table, coord, val|table.set(coord, val))).0;
    Box::new(htr_table)
}

pub mod table {
    use std::collections::HashMap;
    use std::str::FromStr;

    use log::{debug, warn};

    use crate::algs::Algorithm;
    use crate::cube::Cube333;
    use crate::cube::turn::TurnableMut;
    use crate::solver::lookup_table::EmptyVal;
    use crate::solver::moveset::MoveSet;
    use crate::steps::coord::Coord;
    use crate::steps::htr::coords::HTRDRUDCoord;
    use crate::steps::util::Subset;

    pub fn generate_subset_table<
        Mapper,
        Table: EmptyVal,
        Init,
        Getter,
        Setter,
    >(
        move_set: &MoveSet,
        subset: &Subset,
        mapper: &Mapper,
        init: &Init,
        getter: &Getter,
        setter: &Setter,
    ) -> Table
    where
        Mapper: Fn(&Cube333) -> HTRDRUDCoord,
        Init: Fn() -> Table,
        Setter: Fn(&mut Table, HTRDRUDCoord, u8),
        Getter: Fn(&Table, HTRDRUDCoord) -> u8
    {
        let start: Cube333 = Algorithm::from_str(subset.generator).expect("valid subset").into();
        let mut visited = HashMap::new();
        let mut to_check = vec![start.clone()];
        visited.insert(mapper(&start), start);
        while !to_check.is_empty() {
            to_check = pre_gen_coset_0(&move_set, mapper, &mut visited, &to_check);
        }

        let mut to_check = HashMap::new();
        let mut table = init();
        for (start_coord, start_cube) in visited {
            setter(&mut table, start_coord, 0);
            to_check.insert(start_coord, start_cube);
        }
        if to_check.len() > 1 {
            debug!("Found {} variations of the goal state", to_check.len());
        }
        let mut total_checked = 0;
        for depth in 0.. {
            total_checked += to_check.len();
            debug!(
            "Checked {:width$}/{} cubes at depth {depth} (new {})",
            total_checked,
            HTRDRUDCoord::size(),
            to_check.len(),
            width = HTRDRUDCoord::size().to_string().len(),
        );
            to_check = fill_table(&move_set, &mut table, depth, mapper, getter, setter, to_check);
            if to_check.is_empty() {
                break;
            }
        }
        total_checked += to_check.len();
        if total_checked != HTRDRUDCoord::size() {
            warn!(
            "Expected {} cubes in table but got {total_checked}. The coordinate may be malformed",
            HTRDRUDCoord::size()
        );
        }
        table
    }

    fn pre_gen_coset_0<
        Mapper,
    >(
        move_set: &MoveSet,
        mapper: &Mapper,
        visited: &mut HashMap<HTRDRUDCoord, Cube333>,
        to_check: &Vec<Cube333>,
    ) -> Vec<Cube333>
    where
        Mapper: Fn(&Cube333) -> HTRDRUDCoord,
    {
        let mut check_next = vec![];
        for cube in to_check {
            for m in move_set.aux_moves.iter().cloned() {
                let mut cube = cube.clone();
                cube.turn(m);
                let coord = mapper(&cube);
                if visited.contains_key(&coord) {
                    continue;
                }
                visited.insert(coord, cube.clone());
                check_next.push(cube);
            }
        }
        check_next
    }

    fn fill_table<
        Mapper,
        Table: EmptyVal,
        Getter,
        Setter,
    >(
        move_set: &MoveSet,
        table: &mut Table,
        depth: u8,
        mapper: &Mapper,
        getter: &Getter,
        setter: &Setter,
        to_check: HashMap<HTRDRUDCoord, Cube333>,
    ) -> HashMap<HTRDRUDCoord, Cube333>
    where
        Mapper: Fn(&Cube333) -> HTRDRUDCoord,
        Setter: Fn(&mut Table, HTRDRUDCoord, u8),
        Getter: Fn(&Table, HTRDRUDCoord) -> u8
    {
        let mut next_cubes: HashMap<HTRDRUDCoord, Cube333> = HashMap::new();
        for (_coord, cube) in to_check.into_iter() {
            for m in move_set
                .aux_moves
                .into_iter()
                .chain(move_set.st_moves.into_iter())
                .cloned()
            {
                let mut cube = cube.clone();
                cube.turn(m);
                let coord = mapper(&cube);
                let stored = getter(&table, coord);
                if stored == table.empty_val() {
                    setter(table, coord, depth + 1);
                    next_cubes.insert(coord, cube);
                }
            }
        }
        next_cubes
    }
}

pub mod builder {
    use crate::cube::CubeAxis;
    use crate::defs::{NissSwitchType, StepKind};
    use crate::solver_new::dr_subset::DRSubsetStep;
    use crate::solver_new::group::StepGroup;
    use crate::solver_new::step::DFSParameters;
    use crate::steps::step::StepConfig;

    pub struct DRSubsetBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
        _d_dr_axis: Vec<CubeAxis>,
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> DRSubsetBuilderInternal<A, B, C, D> {
        fn convert<const _A: bool, const _B: bool, const _C: bool, const _D: bool>(self) -> DRSubsetBuilderInternal<_A, _B, _C, _D> {
            DRSubsetBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_niss: self._c_niss,
                _d_dr_axis: self._d_dr_axis,
            }
        }
    }

    impl <const B: bool, const C: bool, const D: bool> DRSubsetBuilderInternal<false, B, C, D> {
        pub fn max_length(mut self, max_length: usize) -> DRSubsetBuilderInternal<true, B, C, D> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool, const D: bool> DRSubsetBuilderInternal<A, false, C, D> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> DRSubsetBuilderInternal<A, true, C, D> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const D: bool> DRSubsetBuilderInternal<A, B, false, D> {
        pub fn niss(mut self, niss: NissSwitchType) -> DRSubsetBuilderInternal<A, B, true, D> {
            self._c_niss = niss;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool> DRSubsetBuilderInternal<A, B, C, false> {
        pub fn dr_axis(mut self, eo_axis: Vec<CubeAxis>) -> DRSubsetBuilderInternal<A, B, C, true> {
            self._d_dr_axis = eo_axis;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool> DRSubsetBuilderInternal<A, B, C, D> {
        pub fn build(self) -> StepGroup {
            let dfs = DFSParameters {
                niss_type: self._c_niss,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
                ignore_previous_step_restrictions: true,
            };
            DRSubsetStep::new(dfs, self._d_dr_axis)
        }
    }

    impl DRSubsetBuilderInternal<false, false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 14,
                _b_max_absolute_length: 30,
                _c_niss: NissSwitchType::Before,
                _d_dr_axis: vec![CubeAxis::X, CubeAxis::Y, CubeAxis::Z],
            }
        }
    }

    impl Default for DRSubsetBuilderInternal<false, false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl TryFrom<StepConfig> for DRSubsetBuilderInternal<false, false, false, false> {
        type Error = ();

        fn try_from(value: StepConfig) -> Result<Self, Self::Error> {
            if !value.params.is_empty() {
                return Err(())
            }
            if value.kind != StepKind::DR_4A1_4E {
                return Err(())
            }
            let mut defaults = Self::default();
            if let Some(max) = value.max {
                defaults._a_max_length = max as usize;
            }
            if let Some(abs_max) = value.absolute_max {
                defaults._b_max_absolute_length = abs_max as usize;
            }
            if let Some(niss) = value.niss {
                defaults._c_niss = niss;
            }
            if let Some(variants) = value.substeps {
                let axis: Result<Vec<CubeAxis>, Self::Error> = variants.into_iter()
                    .map(|variant| match variant.to_lowercase().as_str() {
                        "ud" => Ok(CubeAxis::UD),
                        "fb" => Ok(CubeAxis::FB),
                        "lr" => Ok(CubeAxis::LR),
                        _ => Err(()),
                    })
                    .collect();
                defaults._d_dr_axis = axis?;
            }
            Ok(defaults)
        }
    }
}