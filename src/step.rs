use itertools::Itertools;
use crate::moveset::MoveSet;
use crate::{Algorithm, dfs_iter, SearchOptions, stream, Transformation, Turnable};
use crate::cube::Invertible;

pub trait StepVariant<'a, const SC_SIZE: usize, const AUX_SIZE: usize, C>: IsReadyForStep<C> {
    fn move_set(&self) -> &'a MoveSet<SC_SIZE, AUX_SIZE>;
    fn pre_step_trans(&self) -> &'_ Vec<Transformation>;
    fn heuristic(&self, cube: &C) -> u8;
}

pub trait IsReadyForStep<C> {
    fn is_cube_ready(&self, cube: &C) -> bool;
}

pub struct Step<'a, const SC_SIZE: usize, const AUX_SIZE: usize, C> {
    step_variants: Vec<Box<dyn StepVariant<'a, SC_SIZE, AUX_SIZE, C> + 'a>>
}

impl <'a, const SC_SIZE: usize, const AUX_SIZE: usize, C> Step<'a, SC_SIZE, AUX_SIZE, C> {
    pub fn new(step_variants: Vec<Box<dyn StepVariant<'a, SC_SIZE, AUX_SIZE, C> + 'a>>) -> Self {
        Step {
            step_variants
        }
    }
}
pub fn first_step<
    'a,
    'b,
    const SC_SIZE: usize,
    const AUX_SIZE: usize,
    C: Turnable + Invertible + Copy
>(
    step: &'a Step<'b, SC_SIZE, AUX_SIZE, C>, search_opts: SearchOptions, cube: C) -> impl Iterator<Item = Algorithm> + 'a {
    // next_step(vec![].into_iter(), step, search_opts, cube)
    vec![].into_iter()
}

//TODO once we have a better way to merge alg iterators, we should invoke df_search with the full bounds immediately.
//It's not significantly more efficient yet, but in the future it probably will be
pub fn next_step<'a, IN: Iterator<Item=Algorithm> + 'a, const SC_SIZE: usize, const AUX_SIZE: usize, C: Turnable + Invertible + Copy>(algs: IN, step: &'a Step<'a, SC_SIZE, AUX_SIZE, C>, search_opts: SearchOptions, cube: C) -> impl Iterator<Item = Algorithm> + 'a {
    stream::iterated_dfs(algs, move |alg, depth|{
        let result: Box::<dyn Iterator<Item = Algorithm>> = if depth < search_opts.min_moves || depth > search_opts.max_moves {
            Box::new(vec![].into_iter())
        } else {
            let stage_opts = SearchOptions::new(depth, depth, search_opts.niss_type);
            let values = step.step_variants.iter()
                .flat_map(move |variant| dfs_iter(variant.as_ref(), cube.clone(), stage_opts.clone()))
                .flat_map(|iter| iter);
            Box::new(values)
        };
        result
    })
}