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
use crate::steps::{dr, step};
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

pub fn from_step_config<'a, C: 'a>(table: &'a DRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>, {

    let step = if let Some(substeps) = config.substeps {
        let variants: Result<Vec<Vec<Box<dyn StepVariant<C> + 'a>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, [Axis::FB, Axis::LR], [Axis::UD])),
            "fb" | "drfb" => Ok(dr_step_variants(table, [Axis::UD, Axis::LR], [Axis::FB])),
            "lr" | "drlr" => Ok(dr_step_variants(table, [Axis::UD, Axis::FB], [Axis::LR])),

            "eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB, Axis::LR])),
            "eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD, Axis::LR])),
            "eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD, Axis::FB])),

            "drud-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::UD])),
            "drud-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::UD])),
            "drfb-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::FB])),
            "drfb-eolr" => Ok(dr_step_variants(table, [Axis::LR], [Axis::FB])),
            "drlr-eoud" => Ok(dr_step_variants(table, [Axis::UD], [Axis::LR])),
            "drlr-eofb" => Ok(dr_step_variants(table, [Axis::FB], [Axis::LR])),

            x => Err(format!("Invalid DR substep {x}"))
        }).collect();
        let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
        Step::new(variants, "dr")
    } else {
        dr_any(table)
    };

    let search_opts = DefaultStepOptions::new(
        config.min.unwrap_or(0),
        config.max.unwrap_or(12),
        config.niss.unwrap_or(NissType::Before),
        config.quality,
        config.solution_count
    );
    Ok((step, search_opts))
}

fn dr_step_variants<'a, C: 'a, const EOA: usize, const DRA: usize>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA]
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
                (Axis::UD, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X], table, AnyPostStepCheck, "drfb-eoud"))),
                (Axis::UD, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X, Transformation::Z], table, AnyPostStepCheck, "drlr-eoud"))),
                (Axis::FB, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![], table, AnyPostStepCheck, "drud-eofb"))),
                (Axis::FB, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Z], table, AnyPostStepCheck, "drlr-eofb"))),
                (Axis::LR, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y], table, AnyPostStepCheck, "drud-eolr"))),
                (Axis::LR, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{coords::dr::DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y, Transformation::Z], table, AnyPostStepCheck, "drfb-eolr"))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec()
}

pub fn dr<'a, C: 'a, const EOA: usize, const DRA: usize>(
    table: &'a DRPruningTable,
    eo_axis: [Axis; EOA],
    dr_axis: [Axis; DRA],
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = dr_step_variants(table, eo_axis, dr_axis);
    Step::new(step_variants, "dr")
}

pub fn dr_any<'a, C: 'a>(
    table: &'a DRPruningTable,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    dr(table, [Axis::UD, Axis::FB, Axis::LR], [Axis::UD, Axis::FB, Axis::LR])
}

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_transitions(axis_face)
}
