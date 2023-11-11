use std::vec;

use itertools::Itertools;

use crate::defs::*;
use crate::solver::lookup_table::PruningTable;
use crate::solver::moveset::TransitionTable333;
use crate::puzzles::c333::{Cube333, Transformation333, Turn333};
use crate::puzzles::c333::steps::{MoveSet333, Step333};
use crate::puzzles::c333::steps::dr::coords::{DRUDEOFB_SIZE, DRUDEOFBCoord};
use crate::puzzles::c333::steps::eo::coords::EOCoordFB;
use crate::puzzles::cube::{CubeAxis, CubeFace};
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::Direction::*;
use crate::steps::step::{AnyPostStepCheck, DefaultPruningTableStep, DefaultStepOptions, Step, StepVariant};
use crate::steps::step::StepConfig;

pub const HTR_DR_UD_STATE_CHANGE_MOVES: &[Turn333] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
];

pub const HTR_MOVES: &[Turn333] = &[
    Turn333::new(Up, Half),
    Turn333::new(Down, Half),
    Turn333::new(Right, Half),
    Turn333::new(Left, Half),
    Turn333::new(Front, Half),
    Turn333::new(Back, Half),
];

pub const HTR_DR_UD_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: HTR_DR_UD_STATE_CHANGE_MOVES,
    aux_moves: HTR_MOVES,
    transitions: &dr_transitions(Up),
};

pub const DR_UD_EO_FB_STATE_CHANGE_MOVES: &[Turn333] = &[
    Turn333::new(Right, Clockwise),
    Turn333::new(Right, CounterClockwise),
    Turn333::new(Left, Clockwise),
    Turn333::new(Left, CounterClockwise),
];

pub const DR_UD_EO_FB_MOVES: &[Turn333] = &[
    Turn333::new(Up, Clockwise),
    Turn333::new(Up, CounterClockwise),
    Turn333::new(Up, Half),
    Turn333::new(Down, Clockwise),
    Turn333::new(Down, CounterClockwise),
    Turn333::new(Down, Half),
    Turn333::new(Right, Half),
    Turn333::new(Left, Half),
    Turn333::new(Front, Half),
    Turn333::new(Back, Half),
];

pub const DR_UD_EO_FB_MOVESET: MoveSet333 = MoveSet333 {
    st_moves: DR_UD_EO_FB_STATE_CHANGE_MOVES,
    aux_moves: DR_UD_EO_FB_MOVES,
    transitions: &dr_transitions(Left),
};

pub type DRPruningTable = PruningTable<{ DRUDEOFB_SIZE }, DRUDEOFBCoord>;
pub type DRPruningTableStep<'a> = DefaultPruningTableStep<'a, {DRUDEOFB_SIZE}, DRUDEOFBCoord, 2048, EOCoordFB, Turn333, Transformation333, Cube333, TransitionTable333, AnyPostStepCheck>;

pub fn from_step_config(table: &DRPruningTable, config: StepConfig) -> Result<(Step333, DefaultStepOptions), String> {
    let step = if let Some(substeps) = config.substeps {
        let variants: Result<Vec<Vec<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333>>>>, String> = substeps.into_iter().map(|step| match step.to_lowercase().as_str() {
            "ud" | "drud" => Ok(dr_step_variants(table, vec![CubeAxis::FB, CubeAxis::LR], vec![CubeAxis::UD])),
            "fb" | "drfb" => Ok(dr_step_variants(table, vec![CubeAxis::UD, CubeAxis::LR], vec![CubeAxis::FB])),
            "lr" | "drlr" => Ok(dr_step_variants(table, vec![CubeAxis::UD, CubeAxis::FB], vec![CubeAxis::LR])),

            "eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::FB, CubeAxis::LR])),
            "eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::UD, CubeAxis::LR])),
            "eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::UD, CubeAxis::FB])),

            "drud-eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::UD])),
            "drud-eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::UD])),
            "drfb-eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::FB])),
            "drfb-eolr" => Ok(dr_step_variants(table, vec![CubeAxis::LR], vec![CubeAxis::FB])),
            "drlr-eoud" => Ok(dr_step_variants(table, vec![CubeAxis::UD], vec![CubeAxis::LR])),
            "drlr-eofb" => Ok(dr_step_variants(table, vec![CubeAxis::FB], vec![CubeAxis::LR])),

            x => Err(format!("Invalid DR substep {x}"))
        }).collect();
        let variants = variants?.into_iter().flat_map(|v|v).collect_vec();
        Step333::new(variants, StepKind::DR, true)
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

fn dr_step_variants<'a>(table: &'a DRPruningTable, eo_axis: Vec<CubeAxis>, dr_axis: Vec<CubeAxis>) -> Vec<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333> + 'a>> {
    eo_axis
        .into_iter()
        .flat_map(|eo| dr_axis.clone().into_iter().map(move |dr| (eo, dr)))
        .flat_map(move |x| {
            let x: Option<Box<dyn StepVariant<Turn333, Transformation333, Cube333, TransitionTable333>>> = match x {
                (CubeAxis::UD, CubeAxis::FB) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![Transformation333::X], table, AnyPostStepCheck, "fb-eoud"))),
                (CubeAxis::UD, CubeAxis::LR) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![Transformation333::X, Transformation333::Z], table, AnyPostStepCheck, "lr-eoud"))),
                (CubeAxis::FB, CubeAxis::UD) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![], table, AnyPostStepCheck, "ud-eofb"))),
                (CubeAxis::FB, CubeAxis::LR) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![Transformation333::Z], table, AnyPostStepCheck, "lr-eofb"))),
                (CubeAxis::LR, CubeAxis::UD) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![Transformation333::Y], table, AnyPostStepCheck, "ud-eolr"))),
                (CubeAxis::LR, CubeAxis::FB) => Some(Box::new(DRPruningTableStep::new(&DR_UD_EO_FB_MOVESET, vec![Transformation333::Y, Transformation333::Z], table, AnyPostStepCheck, "fb-eolr"))),
                (_eo, _dr) => None,
            };
            x
        })
        .collect_vec()
}

pub fn dr(table: &DRPruningTable, eo_axis: Vec<CubeAxis>, dr_axis: Vec<CubeAxis>, ) -> Step333 {
    let step_variants = dr_step_variants(table, eo_axis, dr_axis);
    Step::new(step_variants, StepKind::DR, true)
}

pub fn dr_any(table: &DRPruningTable) -> Step333 {
    dr(table, vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR], vec![CubeAxis::UD, CubeAxis::FB, CubeAxis::LR])
}

const fn dr_transitions(axis_face: CubeFace) -> [TransitionTable333; 18] {
    crate::puzzles::c333::steps::eo::eo_config::eo_transitions(axis_face)
}
