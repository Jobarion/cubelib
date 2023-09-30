use std::cmp::min;
use std::collections::HashMap;
use std::str::FromStr;
use std::vec;
use itertools::Itertools;
use log::{debug, error, warn};
use crate::algs::Algorithm;
use crate::cli::StepConfig;
use crate::co::COCountUD;
use crate::coords;
use crate::coords::coord::Coord;

use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::eo::EOCoordFB;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation, Turnable};
use crate::cubie::{CornerCubieCube, CubieCube};
use crate::df_search::{NissType};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::dr;
use crate::steps::dr::DRPruningTable;
use crate::steps::dr::DR_UD_EO_FB_MOVESET;
use crate::steps::eo::EOCount;
use crate::steps::htr::HTR_DR_UD_MOVESET;
use crate::steps::step::{DefaultPruningTableStep, PreStepCheck, DefaultStepOptions, Step, StepVariant, AnyPostStepCheck, PostStepCheck};
use crate::stream::distinct_algorithms;

pub struct DRTriggerStepTable<'a> {
    pre_trigger_move_set: &'a MoveSet,
    trigger_move_set: &'a MoveSet,
    pre_trans: Vec<Transformation>,
    table: &'a DRPruningTable,
    trigger_states: HashMap<(u8, u8), u8>, //(co, eolr) - trigger_length
    trigger_variants: Vec<Vec<Move>>,
    name: &'a str,
}
//
// pub fn from_step_config<'a, C: 'a>(table: &'a DRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
//     where
//         DRUDEOFBCoord: for<'x> From<&'x C>,
//         EOCoordFB: for<'x> From<&'x C>, {
//
//     let triggers = config.params
//         .get("triggers")
//         .iter()
//         .flat_map(move |trig|trig.split(","))
//         .filter_map(move |trig|{
//             let alg = Algorithm::from_str(trig.to_uppercase().as_str());
//             match alg {
//                 Err(_) => {
//                     error!("Unable to parse trigger {trig}.");
//                     None
//                 },
//                 Ok(alg) => Some(alg)
//             }
//         })
//         .flat_map(|alg| generate_trigger_variations(alg))
//         .collect_vec();
//
//     let step = if let Some(substeps) = config.substeps {
//         let variants: Result<Vec<Vec<Box<dyn StepVariant<C> + 'a>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
//             "ud" | "drud" => Ok(dr_step_variants(table, [Axis::FB, Axis::LR], [Axis::UD], dr_post_step_check.clone())),
//             "fb" | "drfb" => Ok(dr_step_variants(table, [Axis::UD, Axis::LR], [Axis::FB], dr_post_step_check.clone())),
//             "lr" | "drlr" => Ok(dr_step_variants(table, [Axis::UD, Axis::FB], [Axis::LR], dr_post_step_check.clone())),
//
//             "eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB, Axis::LR], dr_post_step_check.clone())),
//             "eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD, Axis::LR], dr_post_step_check.clone())),
//             "eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD, Axis::FB], dr_post_step_check.clone())),
//
//             "drud-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD], dr_post_step_check.clone())),
//             "drud-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD], dr_post_step_check.clone())),
//             "drfb-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB], dr_post_step_check.clone())),
//             "drfb-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::FB], dr_post_step_check.clone())),
//             "drlr-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::LR], dr_post_step_check.clone())),
//             "drlr-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::LR], dr_post_step_check.clone())),
//
//             x => Err(format!("Invalid DR substep {x}"))
//         }).collect();
//         let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
//         Step::new(variants, "dr")
//     } else {
//         dr_any(table, dr_post_step_check)
//     };
//
//     let search_opts = DefaultStepOptions::new(
//         config.min.unwrap_or(0),
//         config.max.unwrap_or(12),
//         dr_niss_type.unwrap_or(NissType::Before),
//         config.quality,
//         config.solution_count
//     );
//     Ok((step, search_opts))
// }
//
// fn dr_step_variants<'a, C: 'a, const EOA: usize, const DRA: usize>(
//     table: &'a DRPruningTable,
//     eo_axis: [Axis; EOA],
//     dr_axis: [Axis; DRA],
// ) -> Vec<Box<dyn StepVariant<C> + 'a>>
//     where
//         DRUDEOFBCoord: for<'x> From<&'x C>,
//         EOCoordFB: for<'x> From<&'x C>,
// {
//     eo_axis
//         .into_iter()
//         .flat_map(|eo| dr_axis.into_iter().map(move |dr| (eo, dr)))
//         .flat_map(move |x| {
//             let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
//                 (Axis::UD, Axis::FB) => Some(Box::new(DRTriggerStepTable::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X], table, psc.clone(), "drfb-eoud"))),
//                 (Axis::UD, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X, Transformation::Z], table, psc.clone(), "drlr-eoud"))),
//                 (Axis::FB, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![], table, psc.clone(), "drud-eofb"))),
//                 (Axis::FB, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Z], table, psc.clone(), "drlr-eofb"))),
//                 (Axis::LR, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y], table, psc.clone(), "drud-eolr"))),
//                 (Axis::LR, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y, Transformation::Z], table, psc.clone(), "drfb-eolr"))),
//                 (_eo, _dr) => None,
//             };
//             x
//         })
//         .collect_vec()
// }
//
// pub fn dr<'a, C: 'a, const EOA: usize, const DRA: usize, PSC: PostStepCheck<C> + Clone + 'a>(
//     table: &'a DRPruningTable,
//     eo_axis: [Axis; EOA],
//     dr_axis: [Axis; DRA],
//     psc: PSC,
// ) -> Step<'a, C>
// where
//     DRUDEOFBCoord: for<'x> From<&'x C>,
//     EOCoordFB: for<'x> From<&'x C>,
// {
//     let step_variants = dr_step_variants(table, eo_axis, dr_axis, psc);
//     Step::new(step_variants, "dr")
// }
//
// pub fn dr_any<'a, C: 'a, PSC: PostStepCheck<C> + Clone + 'a>(
//     table: &'a DRPruningTable, psc: PSC,
// ) -> Step<'a, C>
// where
//     DRUDEOFBCoord: for<'x> From<&'x C>,
//     EOCoordFB: for<'x> From<&'x C>,
// {
//     dr(table, [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR], psc)
// }

impl<'a, C> DRTriggerStepTable<'a>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>, {

    pub fn new(triggers: Vec<Vec<Move>>) -> Self {
        DRTriggerStepTable {
            pre_trigger_move_set: &HTR_DR_UD_MOVESET,
            trigger_move_set: &DR_UD_EO_FB_MOVESET,
            pre_trans: vec![],
            table,
            trigger_length: 0,
            trigger_variants: vec![],
            name: "eoud",
        }
    }
}

impl<'a, C: COCountUD + EOCount> PreStepCheck<C> for DRTriggerStepTable<'a>
    where
        EOCoordFB: for<'x> From<&'x C>,
{
    fn is_cube_ready(&self, c: &C) -> bool {
        if(EOCoordFB::from(c).val() != 0) {
            return false;
        }
        let eo_count_lr = c.count_bad_edges().2;
        let co_count_ud = COCountUD::co_count(c);


        todo!("Check RZP state")
    }
}

impl<'a, C> StepVariant<C> for DRTriggerStepTable<'a>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>,
{
    fn move_set(&self) -> &'a MoveSet {
        self.pre_trigger_move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &C, depth_left: u8, can_niss: bool) -> u8 {
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
}



impl <'a, CubeParam> PostStepCheck<CubeParam> for DRTriggerStepTable<'a> {
    fn is_solution_admissible(&self, cube: &CubeParam, alg: &Algorithm) -> bool {
        filter_dr_trigger(alg, &self.trigger_variants)
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
    crate::steps::eo::eo_transitions(axis_face)
}
