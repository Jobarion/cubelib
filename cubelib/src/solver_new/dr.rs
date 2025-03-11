use std::cmp::min;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Instant;

use itertools::Itertools;
use log::{debug, error, info, warn};

use crate::algs::Algorithm;
use crate::cube::*;
use crate::cube::turn::{TransformableMut, TurnableMut};
use crate::defs::StepKind;
use crate::solver::lookup_table;
use crate::solver_new::*;
use crate::solver_new::dr::builder::RZPSettings;
use crate::solver_new::group::StepGroup;
use crate::solver_new::htr::HTR_TABLES;
use crate::solver_new::step::*;
use crate::solver_new::thread_util::ToWorker;
use crate::solver_new::util_cube::CubeState;
use crate::steps::coord::Coord;
use crate::steps::dr::co::COCountUD;
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable};
use crate::steps::eo::coords::{BadEdgeCount, EOCoordFB};
use crate::steps::htr::coords::HTRDRUDCoord;
use crate::steps::htr::subsets::{DR_SUBSETS, DRSubsetFilter, Subset};
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

const DRUF_PRE_TRIGGER_ST_MOVES: &[Turn333] = &[
    Turn333::U, Turn333::Ui,
    Turn333::D, Turn333::Di,
];

const DRUF_PRE_TRIGGER_AUX_MOVES: &[Turn333] = &[
    Turn333::U2,
    Turn333::D2,
    Turn333::F2,
    Turn333::B2,
    Turn333::L2,
    Turn333::R2,
];

pub type DRBuilder = builder::DRBuilderInternal<false, false, false, false, false, false, false>;
pub type RZPBuilder = builder::RZPBuilderInternal<false, false, false>;
pub const DRUD_EOFB_MOVESET: MoveSet = MoveSet::new(DRUD_EOFB_ST_MOVES, DRUD_EOFB_AUX_MOVES);
pub const DRUD_PRE_TRIGGER_MOVESET: MoveSet = MoveSet::new(DRUF_PRE_TRIGGER_ST_MOVES, DRUF_PRE_TRIGGER_AUX_MOVES);
pub const DRUD_EOFB_POST_TRIGGER_MOVESET: MoveSet = DRUD_EOFB_MOVESET;

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

    pub fn new_with_triggers(dfs_parameters: DFSParameters, axis: HashMap<CubeAxis, Vec<CubeAxis>>, subsets: Vec<Subset>, triggers: Vec<Algorithm>, rzp_settings: RZPSettings) -> Box<dyn ToWorker + Send + 'static> {
        let mut trigger_variants = vec![];
        let mut trigger_types: HashMap<(u8, u8), usize> = HashMap::new();
        for trigger in triggers.into_iter() {
            let mut cube = Cube333::default();
            for (m, len) in trigger.normal_moves.iter().rev().zip(1..) {
                cube.turn(m.clone());
                if DR_UD_EO_FB_MOVESET.st_moves.contains(m) {
                    let rzp_state = calc_rzp_state(&cube);
                    trigger_types.insert(rzp_state, len);
                    debug!("Registering {}c/{}e trigger with length {}", rzp_state.0, rzp_state.1, len);
                }
            }
            trigger_variants.append(&mut Self::generate_trigger_variations(trigger));
        }

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
                let rzp: Box<dyn ToWorker + Send + 'static> = Box::new(RZPStep {
                    dfs: rzp_settings.dfs.clone(),
                    pre_step_trans: trans.clone(),
                    name: name.clone(),
                });
                let dr_trigger: Box<dyn ToWorker + Send + 'static> = Box::new(DRTriggerStep {
                    table: &DR_TABLE,
                    options: dfs_parameters.clone(),
                    pre_step_trans: trans,
                    post_step_check: psc,
                    name,
                    trigger_variants: trigger_variants.clone(),
                    trigger_types: trigger_types.clone(),
                });
                StepGroup::sequential(vec![rzp, dr_trigger])
            })
            .collect_vec();
        if variants.len() == 1 {
            variants.pop().unwrap()
        } else {
            StepGroup::parallel(variants)
        }
    }

    fn generate_trigger_variations(mut trigger: Algorithm) -> Vec<Vec<Turn333>> {
        if !trigger.inverse_moves.is_empty() {
            error!("Triggers with inverse components are not supported");
            return vec![];
        }
        if let Some(last) = trigger.normal_moves.last() {
            if !last.face.is_on_axis(CubeAxis::LR) || last.dir == Direction::Half {
                warn!("Ignoring DRUD triggers that don't end with R R' L or L'");
                return vec![];
            }
        } else {
            warn!("Ignoring empty triggers");
            return vec![];
        };
        let mut triggers: Vec<Vec<Turn333>> = vec![];
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::UD, Direction::Half));
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::FB, Direction::Half));
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::UD, Direction::Half));
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::FB, Direction::Half));
        trigger.mirror(CubeAxis::LR);
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::UD, Direction::Half));
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::FB, Direction::Half));
        triggers.push(trigger.normal_moves.clone());
        trigger.transform(Transformation333::new(CubeAxis::UD, Direction::Half));
        triggers.push(trigger.normal_moves.clone());

        triggers.into_iter()
            .map(|mut moves| {
                let last = moves.len() - 1;
                moves[last] = Turn333::new(moves[last].face, Direction::Clockwise);
                moves
            })
            .unique()
            .collect_vec()
    }
}

struct DRTriggerStep {
    table: &'static DRPruningTable,
    options: DFSParameters,
    pre_step_trans: Vec<Transformation333>,
    name: String,
    post_step_check: Vec<Box<dyn PostStepCheck + Send + 'static>>,
    trigger_types: HashMap<(u8, u8), usize>,
    trigger_variants: Vec<Vec<Turn333>>,
}

impl PreStepCheck for DRTriggerStep {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        if EOCoordFB::from(cube).val() != 0 {
            return false;
        }
        if DRUDEOFBCoord::from(cube).val() == 0 {
            return true;
        }
        let trigger_state = calc_rzp_state(cube);
        self.trigger_types.contains_key(&trigger_state)
    }
}

impl PostStepCheck for DRTriggerStep {
    fn is_solution_admissible(&self, cube: &Cube333, alg: &Algorithm) -> bool {
        if alg.len() > 0 && !filter_dr_trigger(alg, &self.trigger_variants) {
            false
        } else {
            self.post_step_check.iter()
                .all(|psc| psc.is_solution_admissible(cube, alg))
        }
    }
}

fn filter_dr_trigger(alg: &Algorithm, triggers: &Vec<Vec<Turn333>>) -> bool {
    if alg.len() == 0 {
        return true;
    }
    let mut temp_alg = alg.clone();
    if !temp_alg.normal_moves.is_empty() {
        let last_id = temp_alg.normal_moves.len() - 1;
        let last = temp_alg.normal_moves[last_id];
        temp_alg.normal_moves[last_id] = Turn333::new(last.face, if last.dir == Direction::Half { Direction::Half } else {Direction::Clockwise});
        let normal_match = triggers.iter()
            .any(|trigger|temp_alg.normal_moves.ends_with(trigger));
        if normal_match {
            return true;
        }
    }
    if !temp_alg.inverse_moves.is_empty() {
        let last_id = temp_alg.inverse_moves.len() - 1;
        let last = temp_alg.inverse_moves[last_id];
        temp_alg.inverse_moves[last_id] = Turn333::new(last.face, if last.dir == Direction::Half { Direction::Half } else {Direction::Clockwise});
        let inverse_match = triggers.iter()
            .any(|trigger|temp_alg.inverse_moves.ends_with(trigger));
        if inverse_match {
            return true;
        }
    }
    return false;
}

impl Step for DRTriggerStep {
    fn get_dfs_parameters(&self) -> DFSParameters {
        self.options.clone()
    }

    fn get_moveset(&self, state: &Cube333, depth_left: usize) -> &'_ MoveSet {
        let rzp_state = calc_rzp_state(state);
        if let Some(trigger_length) = self.trigger_types.get(&rzp_state) {
            if *trigger_length >= depth_left {
                &DRUD_EOFB_MOVESET
            } else {
                &DRUD_PRE_TRIGGER_MOVESET
            }
        } else {
            &DRUD_PRE_TRIGGER_MOVESET
        }
    }

    fn heuristic(&self, state: &Cube333, can_niss_switch: bool, _: usize) -> usize {
        let coord = DRUDEOFBCoord::from(state);
        let heuristic = self.table.get(coord);
        if can_niss_switch {
            min(1, heuristic as usize)
        } else {
            heuristic as usize
        }
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_step_trans
    }

    fn get_name(&self) -> (StepKind, String) {
        (StepKind::DR, self.name.clone())
    }
}

struct RZPStep {
    dfs: DFSParameters,
    pre_step_trans: Vec<Transformation333>,
    name: String
}

impl PreStepCheck for RZPStep {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        cube.count_bad_edges_fb() == 0
    }
}

impl PostStepCheck for RZPStep {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm) -> bool {
        true
    }
}

impl Step for RZPStep {
    fn get_dfs_parameters(&self) -> DFSParameters {
        self.dfs.clone()
    }

    fn get_moveset(&self, _: &Cube333, _: usize) -> &'_ MoveSet {
        &DRUD_EOFB_MOVESET
    }

    fn heuristic(&self, _: &Cube333, _: bool, depth_left: usize) -> usize {
        depth_left
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_step_trans
    }

    fn get_name(&self) -> (StepKind, String) {
        (StepKind::RZP, self.name.clone())
    }
}

fn calc_rzp_state(cube: &Cube333) -> (u8, u8) {
    let eo_count_lr = cube.edges.count_bad_edges_lr();
    let co_count_ud = COCountUD::co_count(cube);
    (co_count_ud, eo_count_lr as u8)
}

fn gen_dr() -> DRPruningTable {
    info!("Generating DR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET,
                                       &|c: &Cube333| DRUDEOFBCoord::from(c),
                                       &|| DRPruningTable::new(false),
                                       &|table, coord|table.get(coord),
                                       &|table, coord, val|table.set(coord, val));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

impl Cube333 {
    pub fn get_dr_subset(&self) -> Option<Subset>{
        let state = self.get_cube_state();
        match state {
            CubeState::Scrambled | CubeState::EO(_) => None,
            CubeState::DR(axis) => {
                let mut cube = self.clone();
                cube.transform(match axis[0] {
                    CubeAxis::UD => Transformation333::Y,
                    CubeAxis::FB => Transformation333::X,
                    CubeAxis::LR => Transformation333::Z,
                });
                Some(DR_SUBSETS[HTR_TABLES.1.get(HTRDRUDCoord::from(&cube)) as usize])
            }
            _ => Some(DR_SUBSETS[0])
        }
    }
}

mod builder {
    use std::collections::HashMap;

    use crate::algs::Algorithm;
    use crate::cube::CubeAxis;
    use crate::defs::NissSwitchType;
    use crate::solver_new::dr::{DRStep, RZPBuilder};
    use crate::solver_new::step::DFSParameters;
    use crate::solver_new::thread_util::ToWorker;
    use crate::steps::util::Subset;

    pub struct DRBuilderInternal<const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const F: bool, const G: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
        _d_dr_eo_axis: HashMap<CubeAxis, Vec<CubeAxis>>,
        _e_subsets: Vec<Subset>,
        _f_triggers: Vec<Algorithm>,
        _g_rzp_options: Option<RZPSettings>,
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
        pub fn triggers(mut self, triggers: Vec<Algorithm>) -> DRBuilderInternal<A, B, C, D, E, true, G> {
            self._f_triggers = triggers;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool, const F: bool> DRBuilderInternal<A, B, C, D, E, F, false> {
        pub fn rzp<T: Into<RZPSettings>>(mut self, rzp: T) -> DRBuilderInternal<A, B, C, D, E, F, true> {
            self._g_rzp_options = Some(rzp.into());
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool, const D: bool, const E: bool> DRBuilderInternal<A, B, C, D, E, true, true> {
        pub fn build(self) -> Box<dyn ToWorker + Send + 'static> {
            let dfs = DFSParameters {
                niss_type: self._c_niss,
                min_moves: 0,
                max_moves: self._a_max_length,
                absolute_max_moves: Some(self._b_max_absolute_length),
            };
            DRStep::new_with_triggers(dfs, self._d_dr_eo_axis, self._e_subsets, self._f_triggers, self._g_rzp_options.expect("Guaranteed by the type system"))
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

    pub struct RZPSettings {
        pub(crate) dfs: DFSParameters
    }

    pub struct RZPBuilderInternal<const A: bool, const B: bool, const C: bool> {
        _a_max_length: usize,
        _b_max_absolute_length: usize,
        _c_niss: NissSwitchType,
    }

    impl <const A: bool, const B: bool, const C: bool> RZPBuilderInternal<A, B, C> {
        fn convert<const _A: bool, const _B: bool, const _C: bool>(self) -> RZPBuilderInternal<_A, _B, _C> {
            RZPBuilderInternal {
                _a_max_length: self._a_max_length,
                _b_max_absolute_length: self._b_max_absolute_length,
                _c_niss: self._c_niss,
            }
        }
    }

    impl <const B: bool, const C: bool> RZPBuilderInternal<false, B, C> {
        pub fn max_length(mut self, max_length: usize) -> RZPBuilderInternal<true, B, C> {
            self._a_max_length = max_length;
            self.convert()
        }
    }

    impl <const A: bool, const C: bool> RZPBuilderInternal<A, false, C> {
        pub fn max_absolute_length(mut self, max_absolute_length: usize) -> RZPBuilderInternal<A, true, C> {
            self._b_max_absolute_length = max_absolute_length;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool> RZPBuilderInternal<A, B, false> {
        pub fn niss(mut self, niss: NissSwitchType) -> RZPBuilderInternal<A, B, true> {
            self._c_niss = niss;
            self.convert()
        }
    }

    impl <const A: bool, const B: bool, const C: bool> RZPBuilderInternal<A, B, C> {
        pub fn build(self) -> RZPSettings {
            RZPSettings {
                dfs: DFSParameters {
                    niss_type: self._c_niss,
                    min_moves: 0,
                    max_moves: self._a_max_length,
                    absolute_max_moves: Some(self._b_max_absolute_length),
                }
            }
        }
    }

    impl RZPBuilderInternal<false, false, false> {
        pub fn new() -> Self {
            Self {
                _a_max_length: 3,
                _b_max_absolute_length: 7,
                _c_niss: NissSwitchType::Never,
            }
        }
    }

    impl Default for RZPBuilderInternal<false, false, false> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl <const A: bool, const B: bool, const C: bool> Into<RZPSettings> for RZPBuilderInternal<A, B, C> {
        fn into(self) -> RZPSettings {
            self.build()
        }
    }
}