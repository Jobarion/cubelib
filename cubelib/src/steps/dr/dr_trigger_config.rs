use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;
use std::vec;

use itertools::Itertools;
use log::{debug, error};

use crate::algs::Algorithm;
use crate::co::COCountUD;
use crate::steps::coord::Coord;
use crate::steps::dr::coords::DRUDEOFBCoord;
use crate::steps::eo::coords::EOCoordFB;
use crate::cube::{Axis, Face, Move, NewSolved, Transformation, Turnable};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::CubieCube;
use crate::defs::*;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::dr::dr_config::DR_UD_EO_FB_MOVESET;
use crate::steps::dr::dr_config::DRPruningTable;
use crate::steps::eo::eo_config::EOCount;
use crate::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

pub struct DRTriggerStepTable<'a, CubeParam> {
    pre_trigger_move_set: &'a MoveSet,
    trigger_move_set: &'a MoveSet,
    pre_trans: Vec<Transformation>,
    table: &'a DRPruningTable,
    trigger_types: HashMap<(u8, u8), u8>, //(co, eolr) - trigger_length
    trigger_variants: Vec<Vec<Move>>,
    name: &'a str,
    _c: PhantomData<CubeParam>
}

pub fn from_step_config<'a, C: 'a + EOCount + COCountUD + Display>(table: &'a DRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>, {

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
        let variants: Result<Vec<Vec<Box<dyn StepVariant<C> + 'a>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, [Axis::FB, Axis::LR], [Axis::UD], triggers.clone())),
            "fb" | "drfb" => Ok(dr_step_variants(table, [Axis::UD, Axis::LR], [Axis::FB], triggers.clone())),
            "lr" | "drlr" => Ok(dr_step_variants(table, [Axis::UD, Axis::FB], [Axis::LR], triggers.clone())),

            "eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB, Axis::LR], triggers.clone())),
            "eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD, Axis::LR], triggers.clone())),
            "eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD, Axis::FB], triggers.clone())),

            "drud-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD], triggers.clone())),
            "drud-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD], triggers.clone())),
            "drfb-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB], triggers.clone())),
            "drfb-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::FB], triggers.clone())),
            "drlr-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::LR], triggers.clone())),
            "drlr-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::LR], triggers.clone())),

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
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn dr<'a, C: 'a + COCountUD + EOCount + Display, const EOA: usize, const DRA: usize>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
    triggers: Vec<Algorithm>,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = dr_step_variants(table, eo_axis, dr_axis, triggers);
    Step::new(step_variants, StepKind::DR, true)
}

pub fn dr_any<'a, C: 'a + COCountUD + EOCount + Display>(
    table: &'a DRPruningTable,
    triggers: Vec<Algorithm>,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    dr(table, [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR], triggers)
}

fn dr_step_variants<'a, C: 'a + COCountUD + EOCount + Display, const EOA: usize, const DRA: usize>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
    triggers: Vec<Algorithm>,
) -> Vec<Box<dyn StepVariant<C> + 'a>>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>,
{
    eo_axis
        .into_iter()
        .flat_map(|eo| dr_axis.into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                (Axis::UD, Axis::FB) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation::X], table, triggers.clone(), "fb-eoud"))),
                (Axis::UD, Axis::LR) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation::X, Transformation::Z], table, triggers.clone(), "lr-eoud"))),
                (Axis::FB, Axis::UD) => Some(Box::new(DRTriggerStepTable::new(vec![], table, triggers.clone(), "ud-eofb"))),
                (Axis::FB, Axis::LR) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation::Z], table, triggers.clone(), "lr-eofb"))),
                (Axis::LR, Axis::UD) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation::Y], table, triggers.clone(), "ud-eolr"))),
                (Axis::LR, Axis::FB) => Some(Box::new(DRTriggerStepTable::new(vec![Transformation::Y, Transformation::Z], table, triggers.clone(), "fb-eolr"))),
                _ => None,
            };
            x
        })
        .collect_vec()
}

impl<'a, CubeParam> DRTriggerStepTable<'a, CubeParam>
where
    DRUDEOFBCoord: for<'x> From<&'x CubeParam>,
    EOCoordFB: for<'x> From<&'x CubeParam>, {

    fn new(pre_trans: Vec<Transformation>, table: &'a DRPruningTable, triggers: Vec<Algorithm>, name: &'a str) -> Self {
        let mut trigger_variants = vec![];
        let mut trigger_types: HashMap<(u8, u8), u8> = HashMap::new();
        for trigger in triggers.into_iter() {
            let mut cube = CubieCube::new_solved();
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
            name,
            _c: PhantomData::default()
        }
    }
}

fn calc_rzp_state<C: EOCount + COCountUD>(cube: &C) -> (u8, u8) {
    let eo_count_lr = cube.count_bad_edges().2;
    let co_count_ud = COCountUD::co_count(cube);
    (co_count_ud, eo_count_lr)
}

impl<'a, CubeParam: COCountUD + EOCount> PreStepCheck<CubeParam> for DRTriggerStepTable<'a, CubeParam>
    where
        EOCoordFB: for<'x> From<&'x CubeParam>,
{
    fn is_cube_ready(&self, c: &CubeParam) -> bool {
        if EOCoordFB::from(c).val() != 0 {
            return false;
        }
        let trigger_state = calc_rzp_state(c);
        self.trigger_types.contains_key(&trigger_state)
    }
}

impl <'a, CubeParam> PostStepCheck<CubeParam> for DRTriggerStepTable<'a, CubeParam> {
    fn is_solution_admissible(&self, _: &CubeParam, alg: &Algorithm) -> bool {
        filter_dr_trigger(alg, &self.trigger_variants)
    }
}

impl<'a, CubeParam: COCountUD + EOCount + Display> StepVariant<CubeParam> for DRTriggerStepTable<'a, CubeParam>
    where
        DRUDEOFBCoord: for<'x> From<&'x CubeParam>,
        EOCoordFB: for<'x> From<&'x CubeParam>,
{
    fn move_set(&self, cube: &CubeParam, depth_left: u8) -> &'a MoveSet {
        let rzp_state = calc_rzp_state(cube);
        if let Some(trigger_length) = self.trigger_types.get(&rzp_state) {
            if *trigger_length >= depth_left {
                // println!("{cube}");
                // println!("{}", self.heuristic(cube, depth_left, false));
                // panic!();
                // println!(                "Trigger moves!");
                self.trigger_move_set
            } else {
                self.pre_trigger_move_set
            }
        } else {
            self.pre_trigger_move_set
        }
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &CubeParam, _: u8, can_niss: bool) -> u8 {
        if can_niss {
            1
        } else {
            let coord = DRUDEOFBCoord::from(cube);
            self.table.get(coord).expect("Expected table to be filled")
        }
    }

    fn name(&self) -> &str {
        self.name
    }

    fn is_half_turn_invariant(&self) -> bool {
        true
    }
}

fn generate_trigger_variations(mut trigger: Algorithm) -> Vec<Vec<Move>> {
    if !trigger.inverse_moves.is_empty() {
        error!("Triggers with inverse components are not supported");
        return vec![];
    }
    if let Some(last) = trigger.normal_moves.last() {
        if !last.0.is_on_axis(Axis::LR) || last.1 == Half {
            error!("DRUD triggers should end with R R' L or L'");
            return vec![];
        }
    } else {
        error!("Empty triggers do not make sense");
        return vec![];
    };
    let mut triggers: Vec<Vec<Move>> = vec![];
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::FB, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::FB, Half));
    trigger.mirror(Axis::LR);
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::UD, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::FB, Half));
    triggers.push(trigger.normal_moves.clone());
    trigger.transform(Transformation(Axis::UD, Half));
    triggers.push(trigger.normal_moves.clone());

    triggers.into_iter()
        .map(|mut moves| {
            let last = moves.len() - 1;
            moves[last] = Move(moves[last].0, Clockwise);
            moves
        })
        .unique()
        .collect_vec()
}
pub fn filter_dr_trigger(alg: &Algorithm, triggers: &Vec<Vec<Move>>) -> bool {
    if alg.len() == 0 {
        return true;
    }
    let mut temp_alg = alg.clone();
    if !temp_alg.normal_moves.is_empty() {
        let last_id = temp_alg.normal_moves.len() - 1;
        let last = temp_alg.normal_moves[last_id];
        temp_alg.normal_moves[last_id] = Move(last.0, if last.1 == Half { Half } else {Clockwise});
        let normal_match = triggers.iter()
            .any(|trigger|temp_alg.normal_moves.ends_with(trigger));
        if normal_match {
            return true;
        }
    }
    if !temp_alg.inverse_moves.is_empty() {
        let last_id = temp_alg.inverse_moves.len() - 1;
        let last = temp_alg.inverse_moves[last_id];
        temp_alg.inverse_moves[last_id] = Move(last.0, if last.1 == Half { Half } else {Clockwise});
        let inverse_match = triggers.iter()
            .any(|trigger|temp_alg.inverse_moves.ends_with(trigger));
        if inverse_match {
            return true;
        }
    }
    return false;
}

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_config::eo_transitions(axis_face)
}
