use crate::algs::{Algorithm, Solution};

use crate::coord::Coord;
use crate::cube::{ApplyAlgorithm, Invertible, Transformation, Turnable};
use crate::df_search::{dfs_iter, SearchOptions};
use crate::moveset::MoveSet;
use crate::stream;
use std::cmp::{min};

use std::marker::PhantomData;

pub trait StepVariant<const SC_SIZE: usize, const AUX_SIZE: usize, CubeParam>:
    IsReadyForStep<CubeParam>
{
    fn move_set(&self) -> &'_ MoveSet<SC_SIZE, AUX_SIZE>;
    fn pre_step_trans(&self) -> &'_ Vec<Transformation>;
    fn heuristic(&self, cube: &CubeParam) -> u8;
    fn name(&self) -> &str;
}

pub trait IsReadyForStep<CubeParam> {
    fn is_cube_ready(&self, cube: &CubeParam) -> bool;
}

pub struct Step<'a, const SC_SIZE: usize, const AUX_SIZE: usize, CubeParam> {
    step_variants: Vec<Box<dyn StepVariant<SC_SIZE, AUX_SIZE, CubeParam> + 'a>>,
}

impl<'a, const SC_SIZE: usize, const AUX_SIZE: usize, CubeParam: 'a>
    Step<'a, SC_SIZE, AUX_SIZE, CubeParam>
{
    pub fn new(
        step_variants: Vec<Box<dyn StepVariant<SC_SIZE, AUX_SIZE, CubeParam> + 'a>>,
    ) -> Self {
        Step { step_variants }
    }

    pub fn new_basic<const C_SIZE: usize, CoordParam: Coord<C_SIZE> + 'a>(
        name: &'static str,
        moves: MoveSet<SC_SIZE, AUX_SIZE>,
    ) -> Self
    where
        CoordParam: for<'x> From<&'x CubeParam>,
    {
        let variant: NoHeuristicStep<SC_SIZE, AUX_SIZE, C_SIZE, CoordParam, CubeParam> =
            NoHeuristicStep {
                moves,
                trans: vec![],
                name,
                _c: PhantomData::default(),
                _cube: PhantomData::default(),
            };
        Step {
            step_variants: vec![Box::new(variant)],
        }
    }
}

struct NoHeuristicStep<
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    const C_SIZE: usize,
    CoordParam: Coord<C_SIZE>,
    CubeParam,
> where
    CoordParam: for<'x> From<&'x CubeParam>,
{
    moves: MoveSet<SC_SIZE, AUX_SIZE>,
    trans: Vec<Transformation>,
    name: &'static str,
    _c: PhantomData<CoordParam>,
    _cube: PhantomData<CubeParam>,
}

impl<
        const SC_SIZE: usize,
        const AUX_SIZE: usize,
        const C_SIZE: usize,
        CoordParam: Coord<C_SIZE>,
        CubeParam,
    > IsReadyForStep<CubeParam>
    for NoHeuristicStep<SC_SIZE, AUX_SIZE, C_SIZE, CoordParam, CubeParam>
where
    CoordParam: for<'x> From<&'x CubeParam>,
{
    fn is_cube_ready(&self, _cube: &CubeParam) -> bool {
        true
    }
}

impl<
        const SC_SIZE: usize,
        const AUX_SIZE: usize,
        const C_SIZE: usize,
        CoordParam: Coord<C_SIZE>,
        CubeParam,
    > StepVariant<SC_SIZE, AUX_SIZE, CubeParam>
    for NoHeuristicStep<SC_SIZE, AUX_SIZE, C_SIZE, CoordParam, CubeParam>
where
    CoordParam: for<'x> From<&'x CubeParam>,
{
    fn move_set(&self) -> &'_ MoveSet<SC_SIZE, AUX_SIZE> {
        &self.moves
    }

    fn pre_step_trans(&self) -> &'_ Vec<Transformation> {
        &self.trans
    }

    fn heuristic(&self, cube: &CubeParam) -> u8 {
        min(CoordParam::from(cube).val(), 1) as u8
    }

    fn name(&self) -> &str {
        self.name
    }
}

pub fn first_step<
    'a,
    'b,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    CubeParam: Turnable + Invertible + Copy,
>(
    step: &'a Step<'b, SC_SIZE, AUX_SIZE, CubeParam>,
    search_opts: SearchOptions,
    cube: CubeParam,
) -> impl Iterator<Item = Solution> + 'a {
    next_step(vec![Solution::new()].into_iter(), step, search_opts, cube)
}

//TODO once we have a better way to merge alg iterators, we should invoke df_search with the full bounds immediately.
//It's not significantly more efficient yet, but in the future it probably will be
pub fn next_step<
    'a,
    'b,
    IN: Iterator<Item = Solution> + 'a,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    CubeParam: Turnable + Invertible + Copy,
>(
    algs: IN,
    step: &'a Step<'b, SC_SIZE, AUX_SIZE, CubeParam>,
    search_opts: SearchOptions,
    cube: CubeParam,
) -> impl Iterator<Item = Solution> + 'a {
    stream::iterated_dfs(algs, move |solution, depth| {
        let result: Box<dyn Iterator<Item = Solution>> =
            if depth < search_opts.min_moves || depth > search_opts.max_moves {
                Box::new(vec![].into_iter())
            } else {
                let mut cube = cube.clone();
                let alg: Algorithm = solution.clone().into();
                cube.apply_alg(&alg);
                let stage_opts = SearchOptions::new(depth, depth, search_opts.niss_type);
                let previous_normal = alg.normal_moves.last().cloned();
                let previous_inverse = alg.inverse_moves.last().cloned();

                //Only allow the first variant to use the empty solution, otherwise we get lots of duplicates
                let values = step
                    .step_variants
                    .iter()
                    .zip(0..)
                    .take_while(move |(_step, step_id)| depth > 0 || *step_id == 0)
                    .map(move |(step, _step_id)| {
                        // println!("Trying step {} at depth {}", step.name(), depth);
                        step
                    })
                    .flat_map(move |step_variant| {
                        dfs_iter(
                            step_variant.as_ref(),
                            cube,
                            stage_opts.clone(),
                            previous_normal,
                            previous_inverse,
                        )
                        .map(|alg| (step_variant.name(), alg))
                    })
                    .flat_map(|(name, iter)| iter.map(move |alg| (name, alg)))
                    .map(move |(step_name, step_alg)| {
                        let mut sol = solution.clone();
                        sol.add_step(step_name.to_string(), step_alg);
                        sol
                    });
                Box::new(values)
            };
        result
    })
}
