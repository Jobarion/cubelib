use itertools::Itertools;
use log::{debug, error, warn};
use tinyset::Set64;
use crate::algs::Algorithm;
use crate::defs::*;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{MoveSet333, Step333};
use crate::puzzles::c333::steps::dr::coords::DRUDEOFBCoord;
pub use crate::puzzles::c333::steps::dr::dr_config::HTR_DR_UD_MOVESET;
use crate::puzzles::c333::steps::htr::coords::{HTRDRUD_SIZE, HTRDRUDCoord};
use crate::puzzles::c333::steps::htr::subsets::Subset;
use crate::puzzles::c333::util::expand_subset_name;
use crate::puzzles::cube::CubeAxis;
use crate::puzzles::cube::Direction::*;
use crate::solver::lookup_table::{LookupTable, NissLookupTable};
use crate::solver::moveset::TransitionTable333;
use crate::steps::coord::Coord;
use crate::steps::step::{DefaultStepOptions, PostStepCheck, PreStepCheck, Step, StepVariant};
use crate::steps::step::StepConfig;

pub type HTRPruningTable = NissLookupTable<{ HTRDRUD_SIZE }, HTRDRUDCoord>;
pub type HTRSubsetTable = LookupTable<{ HTRDRUD_SIZE }, HTRDRUDCoord>;

pub struct HTRPruningTableStep<'a, PSC: PreStepCheck<Turn333, Transformation333, Cube333>> {
    move_set: &'a MoveSet333,
    pre_trans: Vec<Transformation333>,
    table: &'a HTRPruningTable,
    pre_step_check: PSC,
    name: &'a str,
}

impl <'a>HTRPruningTableStep<'a, DRUDEOFBPreStepCheck> {
    pub fn new(move_set: &'a MoveSet333,
               pre_trans: Vec<Transformation333>,
               table: &'a HTRPruningTable,
               name: &'a str) -> Self {
        HTRPruningTableStep {
            move_set,
            pre_trans,
            table,
            pre_step_check: DRUDEOFBPreStepCheck{},
            name,
        }
    }
}

impl <'a>HTRPruningTableStep<'a, HTRSubsetPreStepCheck<'a>> {
    pub fn new_subset(move_set: &'a MoveSet333,
                      pre_trans: Vec<Transformation333>,
                      table: &'a HTRPruningTable,
                      subset_table: &'a HTRSubsetTable,
                      subsets: &Vec<(Subset, u8)>,
                      name: &'a str) -> Self {
        let mut subset_set = Set64::new();
        for (_, id) in subsets {
            subset_set.insert(id.clone());
        }
        HTRPruningTableStep {
            move_set,
            pre_trans,
            table,
            pre_step_check: HTRSubsetPreStepCheck(subset_table, subset_set),
            name,
        }
    }
}

struct DRUDEOFBPreStepCheck;

impl PreStepCheck<Turn333, Transformation333, Cube333> for DRUDEOFBPreStepCheck {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        DRUDEOFBCoord::from(cube).val() == 0
    }
}

struct HTRSubsetPreStepCheck<'a>(&'a HTRSubsetTable, Set64<u8>);

impl PreStepCheck<Turn333, Transformation333, Cube333> for HTRSubsetPreStepCheck<'_> {
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        if DRUDEOFBCoord::from(cube).val() != 0 {
            return false;
        }
        let subset_id = self.0.get(HTRDRUDCoord::from(cube));
        self.1.contains(subset_id)
    }
}

impl<'a, PSC: PreStepCheck<Turn333, Transformation333, Cube333>> PreStepCheck<Turn333, Transformation333, Cube333> for HTRPruningTableStep<'a, PSC>
{
    fn is_cube_ready(&self, cube: &Cube333) -> bool {
        self.pre_step_check.is_cube_ready(cube)
    }
}

impl<'a, PSC: PreStepCheck<Turn333, Transformation333, Cube333>> PostStepCheck<Turn333, Transformation333, Cube333> for HTRPruningTableStep<'a, PSC> {
    fn is_solution_admissible(&self, _: &Cube333, _: &Algorithm<Turn333>) -> bool {
        true
    }
}

impl<'a, PSC: PreStepCheck<Turn333, Transformation333, Cube333>> StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> for HTRPruningTableStep<'a, PSC>
{
    fn move_set(&self, _: &Cube333, _: u8) -> &'a MoveSet333 {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation333> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &Cube333, _: u8, can_niss: bool) -> u8 {
        let coord = HTRDRUDCoord::from(cube);
        let (val, niss) = self.table.get(coord);
        if can_niss && val != 0 {
            niss
        } else {
            val
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

pub fn from_step_config<'a>(table: &'a HTRPruningTable, subset_table: &'a HTRSubsetTable, config: StepConfig) -> Result<(Step333<'a>, DefaultStepOptions), String> {
    let subsets = config.params.get("subsets")
        .map(|x|x.split(",").map(|x|x.to_string()).collect_vec());

    let step = if let Some(substeps) = config.substeps {
        let axis: Result<Vec<CubeAxis>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "htrud" | "ud" => Ok(CubeAxis::UD),
            "htrfb" | "fb" => Ok(CubeAxis::FB),
            "htrlr" | "lr" => Ok(CubeAxis::LR),
            x => Err(format!("Invalid HTR substep {x}"))
        }).collect();
        if let Some(subsets) = subsets {
            htr_subsets(table, subset_table, axis?, &subsets)
        } else {
            htr(table, axis?)
        }
    } else {
        if let Some(subsets) = subsets {
            htr_subsets_any(table, subset_table, &subsets)
        } else {
            htr_any(table)
        }
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
                CubeAxis::UD => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![], table, "ud"))),
                CubeAxis::FB => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, "fb"))),
                CubeAxis::LR => Some(Box::new(HTRPruningTableStep::new(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::HTR, true)
}

pub fn htr_subsets_any<'a>(table: &'a HTRPruningTable, subset_table: &'a HTRSubsetTable, subsets: &Vec<String>) -> Step333<'a> {
    htr_subsets(table, subset_table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR], subsets)
}

pub fn htr_subsets<'a>(table: &'a HTRPruningTable, subset_table: &'a HTRSubsetTable, dr_axis: Vec<CubeAxis>, subsets: &Vec<String>) -> Step333<'a> {
    let matched_subsets = subsets.iter()
        .flat_map(|subset_name|{
            let matched_subsets = expand_subset_name(subset_name.as_str());
            if matched_subsets.is_empty() {
                warn!("Unrecognized subset name {subset_name}, ignoring")
            }
            if matched_subsets.len() == 1 {
                for (subset, _) in matched_subsets.iter() {
                    debug!("Adding subset {subset}");
                }
            } else {
                for (subset, _) in matched_subsets.iter() {
                    debug!("Expanding {subset_name} to subset {subset}");
                }
            }
            matched_subsets.into_iter()
        })
        .collect_vec();
    if matched_subsets.is_empty() {
        error!("Unable to recognize any subset, defaulting to all subsets");
        return htr(table, dr_axis);
    }
    let step_variants = dr_axis
        .into_iter()
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> = match x {
                CubeAxis::UD => Some(Box::new(HTRPruningTableStep::new_subset(&HTR_DR_UD_MOVESET, vec![], table, subset_table, &matched_subsets, "ud"))),
                CubeAxis::FB => Some(Box::new(HTRPruningTableStep::new_subset(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::X, Clockwise)], table, subset_table, &matched_subsets, "fb"))),
                CubeAxis::LR => Some(Box::new(HTRPruningTableStep::new_subset(&HTR_DR_UD_MOVESET, vec![Transformation333::new(CubeAxis::Z, Clockwise)], table, subset_table, &matched_subsets, "lr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, StepKind::HTR, true)
}