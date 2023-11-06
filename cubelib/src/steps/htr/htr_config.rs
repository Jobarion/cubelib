use itertools::Itertools;

use crate::cube::{Axis, Transformation};
use crate::cube::Turn::*;
use crate::defs::*;
use crate::lookup_table::PruningTable;
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
pub use crate::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub type HTRPruningTable = PruningTable<{ HTRDRUD_SIZE }, HTRDRUDCoord>;

pub fn from_step_config<'a, C: 'a>(table: &'a HTRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        HTRDRUDCoord: for<'x> From<&'x C>,
        DRUDEOFBCoord: for<'x> From<&'x C>,
{
    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<Axis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "htrud" | "ud" => Ok(Axis::UD),
            "htrfb" | "fb" => Ok(Axis::FB),
            "htrlr" | "lr" => Ok(Axis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        htr(table, axis?)
    } else {
        htr_any(table)
    };
    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(14),
        config.niss.unwrap_or(NissSwitchType::Before),
        if config.quality == 0 {
            None
        } else {
            config.step_limit.or(Some(config.quality * 1))
        }
    );
    Ok((step, search_opts))
}

pub fn htr_any<'a, C: 'a>(
    table: &'a HTRPruningTable,
) -> Step<'a, C>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    htr(table, vec![Axis::UD, Axis::FB, Axis::LR])
}

pub fn htr<'a, C: 'a>(
    table: &'a HTRPruningTable,
    dr_axis: Vec<Axis>,
) -> Step<'a, C>
where
    HTRDRUDCoord: for<'x> From<&'x C>,
    DRUDEOFBCoord: for<'x> From<&'x C>,
{
    let step_variants = dr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{HTRDRUD_SIZE}, HTRDRUDCoord, {DRUDEOFB_SIZE}, DRUDEOFBCoord, C, AnyPostStepCheck>::new(&HTR_DR_UD_MOVESET, vec![], table, AnyPostStepCheck, "ud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{HTRDRUD_SIZE}, HTRDRUDCoord, {DRUDEOFB_SIZE}, DRUDEOFBCoord, C, AnyPostStepCheck>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, AnyPostStepCheck, "fb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{HTRDRUD_SIZE}, HTRDRUDCoord, {DRUDEOFB_SIZE}, DRUDEOFBCoord, C, AnyPostStepCheck>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, AnyPostStepCheck, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::HTR, true)
}