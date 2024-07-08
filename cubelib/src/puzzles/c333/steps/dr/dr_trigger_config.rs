use std::collections::HashMap;
use std::str::FromStr;
use std::vec;

use itertools::Itertools;
use log::{debug, error};

use crate::algs::Algorithm;
use crate::co::COCountUD;
use crate::puzzles::c333::steps::eo::eo_config::EOCount;
use crate::defs::*;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{MoveSet333, Step333};
use crate::puzzles::c333::steps::dr::coords::DRUDEOFBCoord;
use crate::puzzles::c333::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable, HTR_DR_UD_MOVESET};
use crate::puzzles::c333::steps::eo::coords::EOCoordFB;
use crate::puzzles::cube::CubeAxis;
use crate::puzzles::cube::Direction::{Clockwise, Half};
use crate::puzzles::puzzle::{TransformableMut, TurnableMut};
use crate::steps::coord::Coord;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

pub struct DRTriggerStepTable<'a> {
    pre_trigger_move_set: &'a MoveSet333,
    trigger_move_set: &'a MoveSet333,
    pre_trans: Vec<Transformation333>,
    table: &'a DRPruningTable,
    trigger_types: HashMap<(u8, u8), u8>,
    trigger_variants: Vec<Vec<Turn333>>,
    name: &'a str,
}

pub fn from_step_config(table: &DRPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let triggers = config.params
        .get("triggers")
        .iter()
        .flat_map(move |trig|trig.split(","))
        .filter_map(move |trig|{
            let alg = Algorithm::from_str(trig.to_uppercase().as_str());
            match alg {
                Err(_) => {
                    error!("Unable to parse trigger {trig}.");
                    None
                },
                Ok(alg) => Some(alg)
            }
        })
        .collect_vec();

    let step = if let Some(substeps) = config.substeps {
        let variants: Result<Vec<Vec<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333>>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, vec![CubeAxis::FB, CubeAxis::LR], vec![CubeAxis::UD], triggers.clone())),
            "fb" | "drfb" => Ok(dr_step_variants(table, vec![CubeAxis::UD, CubeAxis::LR], vec![CubeAxis::FB], triggers.clone())),
            "lr" | "drlr" => Ok(dr_step_variants(table, vec![CubeAxis::UD, CubeAxis::FB], vec![CubeAxis::LR], triggers.clone())),

            "eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::FB, CubeAxis::LR], triggers.clone())),
            "eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::UD, CubeAxis::LR], triggers.clone())),
            "eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::UD, CubeAxis::FB], triggers.clone())),

            "drud-eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::UD], triggers.clone())),
            "drud-eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::UD], triggers.clone())),
            "drfb-eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::FB], triggers.clone())),
            "drfb-eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::FB], triggers.clone())),
            "drlr-eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::LR], triggers.clone())),
            "drlr-eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::LR], triggers.clone())),

            x => Err(format!("Invalid DR substep {x}"))
        }).collect();
        let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
        Step::new(variants, StepKind::DR, true)
    } else {
        dr_any(table, triggers)
    };

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(12),
        config.absolute_min,
        config.absolute_max,
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn dr(
    table: &DRPruningTable,
    eo_axis: Vec<CubeAxis>,
    dr_axis: Vec<CubeAxis>,
    triggers: Vec<Algorithm<Turn333>>,
) -> Step333 {
    let step_variants = dr_step_variants(table, eo_axis, dr_axis, triggers);
    Step::new(step_variants, StepKind::DR, true)
}

pub fn dr_any(table: &DRPruningTable, triggers: Vec<Algorithm<Turn333>>) -> Step333 {
    dr(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR], vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR], triggers)
}

fn dr_step_variants<'a>(
    table: &'a DRPruningTable,
    eo_axis: Vec<CubeAxis>,
    dr_axis: Vec<CubeAxis>,
    triggers: Vec<Algorithm<Turn333>>,
) -> Vec<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> {
    eo_axis
        .into_iter()
        .flat_map(|eo| dr_axis.clone().into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333>>> = match x {
                (CubeAxis::UD, CubeAxis::FB) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation333::X], table, triggers.clone(), "fb-eoud"))),
                (CubeAxis::UD, CubeAxis::LR) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation333::X, Transformation333::Z], table, triggers.clone(), "lr-eoud"))),
                (CubeAxis::FB, CubeAxis::UD) => Some(Box::new(DRTriggerStepTable::new(vec![], table, triggers.clone(), "ud-eofb"))),
                (CubeAxis::FB, CubeAxis::LR) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation333::Z], table, triggers.clone(), "lr-eofb"))),
                (CubeAxis::LR, CubeAxis::UD) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation333::Y], table, triggers.clone(), "ud-eolr"))),
                (CubeAxis::LR, CubeAxis::FB) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation333::Y, Transformation333::Z], table, triggers.clone(), "fb-eolr"))),
                _ => None,
            };
            x
        })
        .collect_vec()
}

impl<'a> DRTriggerStepTable<'a> {

    fn new(pre_trans: Vec<Transformation333>, table: &'a DRPruningTable, triggers: Vec<Algorithm<Turn333>>, name: &'a str) -> Self {
        let mut trigger_variants = vec![];
        let mut trigger_types: HashMap<(u8, u8), u8> = HashMap::new();
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
            trigger_variants.append(&mut generate_trigger_variations(trigger));
        }
        // //Manually allow the "empty" trigger
        // debug!("Registering 0c/0e trigger with length 0");
        // trigger_types.insert((0, 0), 0);
        // trigger_variants.push(vec![]);

        DRTriggerStepTable {
            pre_trigger_move_set: &HTR_DR_UD_MOVESET,
            trigger_move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans,
            table,
            trigger_types,
            trigger_variants,
            name
        }
    }
}

fn calc_rzp_state(cube: &Cube333) -> (u8, u8) {
    let eo_count_lr = cube.count_bad_edges().2;
    let co_count_ud = COCountUD::co_count(cube);
    (co_count_ud, eo_count_lr)
}

impl<'a> PreStepCheck<Turn333, Transformation333, Cube333> for DRTriggerStepTable<'a> {
    fn is_cube_ready(&self, c: &Cube333) -> bool {
        if EOCoordFB::from(c).val() != 0 {
            return false;
        }
        let trigger_state = calc_rzp_state(c);
        self.trigger_types.contains_key(&trigger_state)
    }
}

impl <'a> PostStepCheck<Turn333, Transformation333, Cube333> for DRTriggerStepTable<'a> {
    fn is_solution_admissible(&self, _: &Cube333, alg: &Algorithm<Turn333>) -> bool {
        filter_dr_trigger(alg, &self.trigger_variants)
    }
}

impl<'a> StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> for DRTriggerStepTable<'a> {
    fn move_set(&self, cube: &Cube333, depth_left: u8) -> &'a MoveSet333 {
        let rzp_state = calc_rzp_state(cube);
        if let Some(trigger_length) = self.trigger_types.get(&rzp_state) {
            if *trigger_length >= depth_left {
                self.trigger_move_set
            } else {
                self.pre_trigger_move_set
            }
        } else {
            self.pre_trigger_move_set
        }
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &Cube333, _: u8, can_niss: bool) -> u8 {
        if can_niss {
            1
        } else {
            let coord = DRUDEOFBCoord::from(cube);
            self.table.get(coord).0
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

fn generate_trigger_variations(mut trigger: Algorithm<Turn333>) -> Vec<Vec<Turn333>> {
    if !trigger.inverse_moves.is_empty() {
        error!("Triggers with inverse components are not supported");
        return vec![];
    }
    if let Some(last) = trigger.normal_moves.last() {
        if !last.face.is_on_axis(CubeAxis::LR) || last.dir == Half {
            error!("DRUD triggers should end with R R' L or L'");
            return vec![];
        }
    } else {
        error!("Empty triggers do not make sense");
        return vec![];
    };
    let mut triggers: Vec<Vec<Turn333>> = vec![];
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::FB, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::FB, Half));
    trigger.mirror(CubeAxis::LR);
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::FB, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation333::new(CubeAxis::UD, Half));
    triggers.push(trigger.normal_moves.clone());

    triggers.into_iter()
        .map(|mut moves| {
            let last = moves.len() - 1;
            moves[last] = Turn333::new(moves[last].face, Clockwise);
            moves
        })
        .unique()
        .collect_vec()
}
pub fn filter_dr_trigger(alg: &Algorithm<Turn333>, triggers: &Vec<Vec<Turn333>>) -> bool {
    if alg.len() == 0 {
        return true;
    }
    let mut temp_alg = alg.clone();
    if !temp_alg.normal_moves.is_empty() {
        let last_id = temp_alg.normal_moves.len() - 1;
        let last = temp_alg.normal_moves[last_id];
        temp_alg.normal_moves[last_id] = Turn333::new(last.face, if last.dir == Half { Half } else {Clockwise});
        let normal_match = triggers.iter()
            .any(|trigger|temp_alg.normal_moves.ends_with(trigger));
        if normal_match {
            return true;
        }
    }
    if !temp_alg.inverse_moves.is_empty() {
        let last_id = temp_alg.inverse_moves.len() - 1;
        let last = temp_alg.inverse_moves[last_id];
        temp_alg.inverse_moves[last_id] = Turn333::new(last.face, if last.dir == Half { Half } else {Clockwise});
        let inverse_match = triggers.iter()
            .any(|trigger|temp_alg.inverse_moves.ends_with(trigger));
        if inverse_match {
            return true;
        }
    }
    return false;
}