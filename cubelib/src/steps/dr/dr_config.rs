use std::vec;

use itertools::Itertools;

use crate::cube::{Axis, Face, Move, Transformation};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::defs::*;
use crate::lookup_table::PruningTable;
use crate::moveset::{MoveSet, TransitionTable};
use crate::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::steps::eo::coords::EOCoordFB;
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

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
    transitions: dr_transitions(Up),
};

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

pub type DRPruningTable = PruningTable<{ DRUDEOFB_SIZE }, DRUDEOFBCoord>;

pub fn from_step_config<'a, C: 'a>(table: &'a DRPruningTable, config: StepConfig) -> Result<(Step<'a, C>, DefaultStepOptions), String>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>, {

    let step = if let Some(substeps) = config.substeps {
        let variants: Result<Vec<Vec<Box<dyn StepVariant<C> + 'a>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, vec![Axis::FB, Axis::LR], vec![Axis::UD])),
            "fb" | "drfb" => Ok(dr_step_variants(table, vec![Axis::UD, Axis::LR], vec![Axis::FB])),
            "lr" | "drlr" => Ok(dr_step_variants(table, vec![Axis::UD, Axis::FB], vec![Axis::LR])),

            "eoud" => Ok(dr_step_variants(table, vec![Axis::UD], vec![Axis::FB, Axis::LR])),
            "eofb" => Ok(dr_step_variants(table, vec![Axis::FB], vec![Axis::UD, Axis::LR])),
            "eolr" => Ok(dr_step_variants(table, vec![Axis::LR], vec![Axis::UD, Axis::FB])),

            "drud-eofb" => Ok(dr_step_variants(table, vec![Axis::FB], vec![Axis::UD])),
            "drud-eolr" => Ok(dr_step_variants(table, vec![Axis::LR], vec![Axis::UD])),
            "drfb-eoud" => Ok(dr_step_variants(table, vec![Axis::UD], vec![Axis::FB])),
            "drfb-eolr" => Ok(dr_step_variants(table, vec![Axis::LR], vec![Axis::FB])),
            "drlr-eoud" => Ok(dr_step_variants(table, vec![Axis::UD], vec![Axis::LR])),
            "drlr-eofb" => Ok(dr_step_variants(table, vec![Axis::FB], vec![Axis::LR])),

            x => Err(format!("Invalid DR substep {x}"))
        }).collect();
        let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
        Step::new(variants, StepKind::DR, true)
    } else {
        dr_any(table)
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

fn dr_step_variants<'a, C: 'a>(
    table: &'a DRPruningTable,
    eo_axis: Vec<Axis>,
    dr_axis: Vec<Axis>
) -> Vec<Box<dyn StepVariant<C> + 'a>>
    where
        DRUDEOFBCoord: for<'x> From<&'x C>,
        EOCoordFB: for<'x> From<&'x C>,
{
    eo_axis
        .into_iter()
        .flat_map(|eo| dr_axis.clone().into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<C> + 'a>> = match x {
                (Axis::UD, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X], table, AnyPostStepCheck, "fb-eoud"))),
                (Axis::UD, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::X, Transformation::Z], table, AnyPostStepCheck, "lr-eoud"))),
                (Axis::FB, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![], table, AnyPostStepCheck, "ud-eofb"))),
                (Axis::FB, Axis::LR) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Z], table, AnyPostStepCheck, "lr-eofb"))),
                (Axis::LR, Axis::UD) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y], table, AnyPostStepCheck, "ud-eolr"))),
                (Axis::LR, Axis::FB) => Some(Box::new(DefaultPruningTableStep::<{DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, C, AnyPostStepCheck>::new(&DR_UD_EO_FB_MOVESET, vec![Transformation::Y, Transformation::Z], table, AnyPostStepCheck, "fb-eolr"))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec()
}

pub fn dr<'a, C: 'a>(
    table: &'a DRPruningTable,
    eo_axis: Vec<Axis>,
    dr_axis: Vec<Axis>,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    let step_variants = dr_step_variants(table, eo_axis, dr_axis);
    Step::new(step_variants, StepKind::DR, true)
}

pub fn dr_any<'a, C: 'a>(
    table: &'a DRPruningTable,
) -> Step<'a, C>
where
    DRUDEOFBCoord: for<'x> From<&'x C>,
    EOCoordFB: for<'x> From<&'x C>,
{
    dr(table, vec![Axis::UD, Axis::FB, Axis::LR], vec![Axis::UD, Axis::FB, Axis::LR])
}

const fn dr_transitions(axis_face: Face) -> [TransitionTable; 18] {
    crate::steps::eo::eo_config::eo_transitions(axis_face)
}
