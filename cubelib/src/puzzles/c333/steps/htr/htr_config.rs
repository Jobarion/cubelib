use itertools::Itertools;

use crate::defs::*;
use crate::solver::lookup_table::PruningTable;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
pub use crate::puzzles::c333::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::puzzles::c333::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};
use crate::puzzles::c333::steps::Step333;
use crate::puzzles::cube::CubeAxis;
use crate::puzzles::cube::Direction::*;
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub type HTRPruningTable = PruningTable<{ HTRDRUD_SIZE }, HTRDRUDCoord>;
pub type HTRPruningTableStep<'a> = DefaultPruningTableStep<'a, {HTRDRUD_SIZE}, HTRDRUDCoord, {DRUDEOFB_SIZE}, DRUDEOFBCoord, Turn333, Transformation333, Cube333, TransitionTable333, AnyPostStepCheck>;

pub fn from_step_config(table: &HTRPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "htrud" | "ud" => Ok(CubeAxis::UD),
            "htrfb" | "fb" => Ok(CubeAxis::FB),
            "htrlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        htr(table, axis?)
    } else {
        htr_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(14),
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

pub fn htr_any(table: &HTRPruningTable) -> Step333 {
    htr(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

pub fn htr<'a>(table: &'a HTRPruningTable, dr_axis: Vec<CubeAxis>) -> Step333<'a> {
    let step_variants = dr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                CubeAxis::FB => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                CubeAxis::LR => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::HTR, true)
}