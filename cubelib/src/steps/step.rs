use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use log::trace;
use tokio_util::sync::CancellationToken;

use crate::algs::Algorithm;
use crate::defs::*;
use crate::puzzles::puzzle::{ApplyAlgorithm, Puzzle, PuzzleMove, Transformable};
use crate::solver::df_search::dfs_iter;
use crate::solver::lookup_table::LookupTable;
use crate::solver::moveset::{MoveSet, TransitionTable};
use crate::solver::solution::{Solution, SolutionStep};
use crate::solver::stream;
use crate::steps::coord::Coord;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct StepConfig {
    pub kind: StepKind,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub substeps: Option<Vec<String>>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub min: Option<u8>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub max: Option<u8>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub absolute_min: Option<u8>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub absolute_max: Option<u8>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub step_limit: Option<usize>,
    pub quality: usize,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing_if = "Option::is_none"))]
    pub niss: Option<NissSwitchType>,
    pub params: HashMap<String, String>,
}

impl StepConfig {
    pub fn new(kind: StepKind) -> StepConfig {
        StepConfig {
            kind,
            substeps: None,
            min: None,
            max: None,
            absolute_min: None,
            absolute_max: None,
            step_limit: None,
            niss: None,
            quality: 100,
            params: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DefaultStepOptions {
    pub niss_type: NissSwitchType,
    pub min_moves: u8,
    pub max_moves: u8,
    pub absolute_min_moves: Option<u8>,
    pub absolute_max_moves: Option<u8>,
    pub step_limit: Option<usize>,
}

//Shh, don't look at the types below

pub trait StepVariant<Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>>:
    PreStepCheck<Turn, Transformation, PuzzleParam> +
    PostStepCheck<Turn, Transformation, PuzzleParam>
{
    fn move_set(&self, cube: &PuzzleParam, depth_left: u8) -> &'_ MoveSet<Turn, TransTable>;
    fn pre_step_trans(&self) -> &'_ Vec<Transformation>;
    fn heuristic(&self, cube: &PuzzleParam, depth_left: u8, can_niss: bool) -> u8;
    fn name(&self) -> &str;
}

pub trait PreStepCheck<Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>> {
    fn is_cube_ready(&self, cube: &PuzzleParam) -> bool;
}

pub trait PostStepCheck<Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>> {
    fn is_solution_admissible(&self, cube: &PuzzleParam, alg: &Algorithm<Turn>) -> bool;
}

#[derive(Copy, Clone)]
pub struct AnyPostStepCheck;

impl <Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>> PostStepCheck<Turn, Transformation, PuzzleParam> for AnyPostStepCheck {
    fn is_solution_admissible(&self, _: &PuzzleParam, _: &Algorithm<Turn>) -> bool {
        true
    }
}

pub struct DefaultPruningTableStep<
    'a,
    const HC_SIZE: usize,
    HC: Coord<HC_SIZE>,
    const PC_SIZE: usize,
    PC: Coord<PC_SIZE>,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn> + 'static,
    PSC: PostStepCheck<Turn, Transformation, PuzzleParam>
>
    where
        HC: for<'x> From<&'x PuzzleParam>,
        PC: for<'x> From<&'x PuzzleParam>,
{
    move_set: &'a MoveSet<Turn, TransTable>,
    pre_trans: Vec<Transformation>,
    table: &'a LookupTable<HC_SIZE, HC>,
    name: &'a str,
    post_step_checker: PSC,
    _pc: PhantomData<PC>,
    _puzzle: PhantomData<PuzzleParam>,
    _turn: PhantomData<Turn>,
}

impl <'a, const HC_SIZE: usize, HC: Coord<HC_SIZE>, const PC_SIZE: usize, PC: Coord<PC_SIZE>, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>, PSC: PostStepCheck<Turn, Transformation, PuzzleParam>> PreStepCheck<Turn, Transformation, PuzzleParam> for DefaultPruningTableStep<'a, HC_SIZE, HC, PC_SIZE, PC, Turn, Transformation, PuzzleParam, TransTable, PSC>
    where
        HC: for<'x> From<&'x PuzzleParam>,
        PC: for<'x> From<&'x PuzzleParam>, {

    fn is_cube_ready(&self, cube: &PuzzleParam) -> bool {
        PC::from(cube).val() == 0
    }
}

impl <'a, const HC_SIZE: usize, HC: Coord<HC_SIZE>, const PC_SIZE: usize, PC: Coord<PC_SIZE>, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>, PSC: PostStepCheck<Turn, Transformation, PuzzleParam>> PostStepCheck<Turn, Transformation, PuzzleParam> for DefaultPruningTableStep<'a, HC_SIZE, HC, PC_SIZE, PC, Turn, Transformation, PuzzleParam, TransTable, PSC>
    where
        HC: for<'x> From<&'x PuzzleParam>,
        PC: for<'x> From<&'x PuzzleParam>, {

    fn is_solution_admissible(&self, cube: &PuzzleParam, alg: &Algorithm<Turn>) -> bool {
        self.post_step_checker.is_solution_admissible(cube, alg)
    }
}

impl <
    'a,
    const HC_SIZE: usize,
    HC: Coord<HC_SIZE>,
    const PC_SIZE: usize,
    PC: Coord<PC_SIZE>,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
    PSC: PostStepCheck<Turn, Transformation, PuzzleParam>>
StepVariant<Turn, Transformation, PuzzleParam, TransTable> for DefaultPruningTableStep<'a, HC_SIZE, HC, PC_SIZE, PC, Turn, Transformation, PuzzleParam, TransTable, PSC>
where
    HC: for<'x> From<&'x PuzzleParam>,
    PC: for<'x> From<&'x PuzzleParam>, {

    fn move_set(&self, _: &PuzzleParam, _: u8) -> &'_ MoveSet<Turn, TransTable> {
        self.move_set
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.pre_trans
    }

    fn heuristic(&self, cube: &PuzzleParam, _: u8, can_niss: bool) -> u8 {
        if can_niss {
            1
        } else {
            let coord = HC::from(cube);
            self.table.get(coord)
        }
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl <
    'a,
    const HC_SIZE: usize,
    HC: Coord<HC_SIZE>,
    const PC_SIZE: usize,
    PC: Coord<PC_SIZE>,
    PSC: PostStepCheck<Turn, Transformation, PuzzleParam>,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation>,
    TransTable: TransitionTable<Turn>,
>
DefaultPruningTableStep<'a, HC_SIZE, HC, PC_SIZE, PC, Turn, Transformation, PuzzleParam, TransTable, PSC>
    where
        HC: for<'x> From<&'x PuzzleParam>,
        PC: for<'x> From<&'x PuzzleParam>, {

    pub fn new(move_set: &'a MoveSet<Turn, TransTable>,
               pre_trans: Vec<Transformation>,
               table: &'a LookupTable<HC_SIZE, HC>,
               post_step_checker: PSC,
               name: &'a str) -> Self {
        DefaultPruningTableStep {
            move_set,
            pre_trans,
            table,
            name,
            post_step_checker,
            _puzzle: PhantomData::default(),
            _pc: PhantomData::default(),
            _turn: PhantomData::default(),
        }
    }
}

pub struct Step<'a, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>> {
    step_variants: Vec<Box<dyn StepVariant<Turn, Transformation, PuzzleParam, TransTable> + 'a>>,
    is_major: bool,
    kind: StepKind,
}

impl<'a, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation> + 'a, TransTable: TransitionTable<Turn>> Step<'a, Turn, Transformation, PuzzleParam, TransTable> {
    pub fn new(
        step_variants: Vec<Box<dyn StepVariant<Turn, Transformation, PuzzleParam, TransTable> + 'a>>,
        kind: StepKind,
        is_major: bool,
    ) -> Self {
        Step { step_variants, kind, is_major }
    }

    pub fn kind(&self) -> StepKind {
        self.kind.clone()
    }
}

pub fn first_step<
    'a,
    'b,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation> + Display,
    TransTable: TransitionTable<Turn> + 'static,
>(
    step: &'a Step<'b, Turn, Transformation, PuzzleParam, TransTable>,
    search_opts: DefaultStepOptions,
    cube: PuzzleParam,
    cancel_token: CancellationToken
) -> impl Iterator<Item = Solution<Turn>> + 'a {
    next_step(vec![Solution::new()].into_iter(), step, search_opts, cube, cancel_token)
}

//TODO once we have a better way to merge alg iterators, we should invoke df_search with the full bounds immediately.
//It's not significantly more efficient yet, but in the future it probably will be
pub fn next_step<
    'a,
    'b,
    IN: Iterator<Item = Solution<Turn>> + 'a,
    Turn: PuzzleMove + Transformable<Transformation>,
    Transformation: PuzzleMove,
    PuzzleParam: Puzzle<Turn, Transformation> + Display,
    TransTable: TransitionTable<Turn>,
>(
    algs: IN,
    step: &'a Step<'b, Turn, Transformation, PuzzleParam, TransTable>,
    search_opts: DefaultStepOptions,
    cube: PuzzleParam,
    cancel_token: CancellationToken,
) -> impl Iterator<Item = Solution<Turn>> + 'a {
    stream::iterated_dfs(algs, cancel_token, move |solution, depth, cancel_token| {
        let absolute_target_length = solution.len() as u8 + depth;
        let result: Box<dyn Iterator<Item = Solution<Turn>>> =
            if depth < search_opts.min_moves ||
                depth > search_opts.max_moves ||
                search_opts.absolute_min_moves.map(|m| m > absolute_target_length).unwrap_or(false) ||
                search_opts.absolute_max_moves.map(|m| m < absolute_target_length).unwrap_or(false) {
                Box::new(vec![].into_iter())
            } else {
                let mut cube = cube.clone();
                let alg: Algorithm<Turn> = solution.clone().into();
                let ends_on_normal = solution.ends_on_normal();
                cube.apply_alg(&alg);
                let stage_opts = DefaultStepOptions::new(depth, depth, None, None, search_opts.niss_type, search_opts.step_limit);
                let previous_normal = alg.normal_moves.last().cloned();
                let previous_inverse = alg.inverse_moves.last().cloned();

                trace!("Current solution step {}, depth {depth}, {alg}, normal {}, {previous_normal:?}, {previous_inverse:?}", step.kind, solution.ends_on_normal);
                //Only allow the first variant to use the empty solution, otherwise we get lots of duplicates
                let values = step
                    .step_variants
                    .iter()
                    .flat_map(move |step_variant| {
                        dfs_iter(
                            step_variant.as_ref(),
                            cube.clone(),
                            stage_opts.clone(),
                            previous_normal,
                            previous_inverse,
                            ends_on_normal,
                            cancel_token.clone(),
                        )
                        .map(|alg| (step_variant.name(), alg))
                    })
                    .flat_map(|(name, iter)| iter.map(move |alg| (name, alg)))
                    .map(move |(variant_name, step_alg)| {
                        let mut sol = solution.clone();
                        if step.is_major || step_alg.len() > 0 {
                            let sol_step = SolutionStep {
                                kind: step.kind(),
                                alg: step_alg,
                                variant: variant_name.to_string()
                            };
                            sol.add_step(sol_step);
                        }
                        sol
                    });
                Box::new(values)
            };
        result
    })
}
