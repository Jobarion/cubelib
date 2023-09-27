use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cube::{Axis, Face, Move, Transformation};
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::step::{DefaultPruningTableStep, IsReadyForStep, Step, StepVariant};
use itertools::Itertools;
use std::fmt::{Debug};
use crate::cli::StepConfig;
use crate::coords;
use crate::coords::coord::Coord;
use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::htr::HTRDRUDCoord;
use crate::df_search::{NissType, SearchOptions};

pub const HTR_DR_UD_STATE_CHANGE_MOVES: &[Move] = &[
    Move(Up, Clockwise),
    Move(Up, CounterClockwise),
    Move(Down, Clockwise),
    Move(Down, CounterClockwise),
];

pub const HTR_MOVES: &[Move] = &[
    Move(Up, Half),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const HTR_DR_UD_MOVESET: MoveSet = MoveSet {
    st_moves: HTR_DR_UD_STATE_CHANGE_MOVES,
    aux_moves: HTR_MOVES,
    transitions: htr_transitions(Up),
};

pub type HTRPruningTable = PruningTable<{ coords::htr::HTRDRUD_SIZE }, HTRDRUDCoord>;

pub fn from_step_config<'a, C: 'a>(table: &'a HTRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, SearchOptions), String>
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
    let search_opts = SearchOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(14),
        config.niss.unwrap_or(NissType::During),
        config.quality,
        config.solution_count
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
                Axis::UD => Some(Box::new(DefaultPruningTableStep::<{coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![], table, "htr-drud"))),
                Axis::FB => Some(Box::new(DefaultPruningTableStep::<{coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::X, Clockwise)], table, "htr-drfb"))),
                Axis::LR => Some(Box::new(DefaultPruningTableStep::<{coords::htr::HTRDRUD_SIZE}, HTRDRUDCoord, {coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, C>::new(&HTR_DR_UD_MOVESET, vec![Transformation(Axis::Z, Clockwise)], table, "htr-drlr"))),
            };
            x
        })
        .collect_vec();
    Step::new(step_variants, "htr")
}

const fn htr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_transitions(axis_face)
}
