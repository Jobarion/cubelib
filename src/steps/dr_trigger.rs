use std::cmp::min;
use std::str::FromStr;
use std::vec;
use itertools::Itertools;
use log::{debug, error, warn};
use crate::algs::Algorithm;
use crate::cli::StepConfig;
use crate::coords;

use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::eo::EOCoordFB;
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation, Turnable};
use crate::df_search::{NissType};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::dr;
use crate::steps::step::{DefaultPruningTableStep, PreStepCheck, DefaultStepOptions, Step, StepVariant, AnyPostStepCheck, PostStepCheck};
use crate::stream::distinct_algorithms;

pub const DR_UD_EO_FB_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Right, Clockwise),
    Move(Right, CounterClockwise),
    Move(Left, Clockwise),
    Move(Left, CounterClockwise),
];

pub const DR_UD_EO_FB_MOVES: &[Move] = &[
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Up, Half),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const DR_UD_EO_FB_MOVESET: MoveSet = MoveSet {
    st_moves: DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: DR_UD_EO_FB_MOVES,
    transitions: dr_transitions(Left),
};

pub type DRPruningTable = PruningTable<{ coords::dr::DRUDEOFB_SIZE }, DRUDEOFBCoord>;

#[derive(Clone)]
pub struct DRPostStepCheck {
    triggers: Option<Vec<Vec<Move>>>,
    rzp_length: usize,
    rzp_niss_only: bool,
}

impl <CubeParam> PostStepCheck<CubeParam> for DRPostStepCheck {
    fn is_solution_admissible(&self, cube: &CubeParam, alg: &Algorithm) -> bool {
        self.triggers.as_ref().map_or(true, |triggers| dr::filter_dr_rzp_trigger(Axis::UD, alg, self.rzp_length, self.rzp_niss_only, triggers))
    }
}

pub fn from_step_config<'a, C: 'a>(table: &'a DRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
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
        .flat_map(|alg| dr::generate_trigger_variations(alg))
        .collect_vec();

    let mut dr_niss_type = config.niss;
    let dr_post_step_check = if triggers.is_empty() {
        DRPostStepCheck {
            rzp_length: 0,
            rzp_niss_only: false,
            triggers: None
        }
    } else {
        let rzp_niss_only = config.params.get("rzp-niss")
            .and_then(|val| bool::from_str(val)
                //TODO .inspect_err once that is stable
                .map_err(|err| {
                    error!("Unable to parse RZP-NISS value {val}. Falling back to default. '{err}'");
                    err
                })
                .ok()
            );
        if rzp_niss_only.is_some() {
            dr_niss_type = Some(NissType::During);
        }
        let rzp_niss_only = rzp_niss_only.unwrap_or(true);
        let rzp_length = config.params.get("rzp")
            .and_then(|rzp| usize::from_str(rzp)
                //TODO .inspect_err once that is stable
                .map_err(|err| {
                    error!("Unable to parse RZP value {rzp}. Falling back to default. '{err}'");
                    err
                })
                .ok())
            .unwrap_or(2); //Default RZP length
        debug!("Restricting DR to max {rzp_length} move RZP with triggers {:?}", triggers.clone().into_iter().map(|t|Algorithm{normal_moves: t, inverse_moves: vec![]}).collect_vec());
        DRPostStepCheck {
            rzp_length,
            rzp_niss_only,
            triggers: Some(triggers)
        }
    };

    let step = if let Some(substeps) = config.substeps {
        let variants: Result<Vec<Vec<Box<dyn StepVariant<C> + 'a>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, [Axis::FB, Axis::LR], [Axis::UD], dr_post_step_check.clone())),
            "fb" | "drfb" => Ok(dr_step_variants(table, [Axis::UD, Axis::LR], [Axis::FB], dr_post_step_check.clone())),
            "lr" | "drlr" => Ok(dr_step_variants(table, [Axis::UD, Axis::FB], [Axis::LR], dr_post_step_check.clone())),

            "eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB, Axis::LR], dr_post_step_check.clone())),
            "eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD, Axis::LR], dr_post_step_check.clone())),
            "eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD, Axis::FB], dr_post_step_check.clone())),

            "drud-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD], dr_post_step_check.clone())),
            "drud-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD], dr_post_step_check.clone())),
            "drfb-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB], dr_post_step_check.clone())),
            "drfb-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::FB], dr_post_step_check.clone())),
            "drlr-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::LR], dr_post_step_check.clone())),
            "drlr-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::LR], dr_post_step_check.clone())),

            x => Err(format!("Invalid DR substep {x}"))
        }).collect();
        let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
        Step::new(variants, "dr")
    } else {
        dr_any(table, dr_post_step_check)
    };

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(12),
        dr_niss_type.unwrap_or(NissType::Before),
        config.quality,
        config.solution_count
    );
    Ok((step, search_opts))
}

fn dr_step_variants<'a, C: 'a, const EOA: usize, const DRA: usize, PSC: PostStepCheck<C> + Clone + 'a>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
    psc: PSC
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
                (Axis::UD, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X], table, psc.clone(), "drfb-eoud"))),
                (Axis::UD, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X, Transformation::Z], table, psc.clone(), "drlr-eoud"))),
                (Axis::FB, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![], table, psc.clone(), "drud-eofb"))),
                (Axis::FB, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Z], table, psc.clone(), "drlr-eofb"))),
                (Axis::LR, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y], table, psc.clone(), "drud-eolr"))),
                (Axis::LR, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, PSC>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y, Transformation::Z], table, psc.clone(), "drfb-eolr"))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec()
}

pub fn dr<'a, C: 'a, const EOA: usize, const DRA: usize, PSC: PostStepCheck<C> + Clone + 'a>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
    psc: PSC,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = dr_step_variants(table, eo_axis, dr_axis, psc);
    Step::new(step_variants, "dr")
}

pub fn dr_any<'a, C: 'a, PSC: PostStepCheck<C> + Clone + 'a>(
    table: &'a DRPruningTable, psc: PSC,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    dr(table, [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR], psc)
}

pub fn generate_trigger_variations(mut trigger: Algorithm) -> Vec<Vec<Move>> {
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

//TODO if there are lots of triggers using a HashSet could make sense?
pub fn filter_dr_rzp_trigger(dr_axis: Axis, alg: &Algorithm, rzp_length: usize, rzp_niss_only: bool, triggers: &Vec<Vec<Move>>) -> bool {
    if alg.len() == 0 {
        return true;
    }
    let mut temp_alg = alg.clone();
    if !temp_alg.normal_moves.is_empty() {
        let last_id = temp_alg.normal_moves.len() - 1;
        let last = temp_alg.normal_moves[last_id];
        temp_alg.normal_moves[last_id] = Move(last.0, if last.1 == Half { Half } else {Clockwise});
    }
    if !temp_alg.inverse_moves.is_empty() {
        let last_id = temp_alg.inverse_moves.len() - 1;
        let last = temp_alg.inverse_moves[last_id];
        temp_alg.inverse_moves[last_id] = Move(last.0, if last.1 == Half { Half } else {Clockwise});
    }

    filter_drud_rzp_trigger_moves(dr_axis, &temp_alg, rzp_length, rzp_niss_only, triggers)
}

fn filter_drud_rzp_trigger_moves(dr_axis: Axis, alg: &Algorithm, rzp_length: usize, rzp_niss: bool, triggers: &Vec<Vec<Move>>) -> bool {
    let alg_len = alg.len();
    triggers.iter()
        .flat_map(|t| {
            let mut variations = vec![];
            if (!rzp_niss || alg.inverse_moves.len() <= rzp_length) && alg.normal_moves.ends_with(t) {
                // println!("Trigger '{:?}' in normal", t);
                let normal: &[Move] = &alg.normal_moves.as_slice()[0..(alg.normal_moves.len() - t.len())];
                let inv: &[Move] = alg.inverse_moves.as_slice();
                variations.push((inv, normal))
            }
            if (!rzp_niss || alg.normal_moves.len() <= rzp_length) && alg.inverse_moves.ends_with(t) {
                // println!("Trigger '{:?}' in inverse", t);
                let normal: &[Move] = alg.normal_moves.as_slice();
                let inv: &[Move] = &alg.inverse_moves.as_slice()[0..(alg.inverse_moves.len() - t.len())];

                variations.push((normal, inv))
            }
            variations.into_iter()
        })
        .any(|(p1, p2)| {
            let rzp_length = if rzp_niss && !p1.is_empty() {
                0
            } else {
                rzp_length
            };
            // println!("{:?} and then {:?}, skipping {rzp_length}", p1, p2);
            !p1.into_iter()
                .chain(p2.into_iter())
                .skip(rzp_length)
                .filter(|t| !t.0.is_on_axis(dr_axis))
                .any(|t| t.1 != Half)
        })
}

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_transitions(axis_face)
}
