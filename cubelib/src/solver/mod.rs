use std::vec;
use log::debug;
use crate::puzzles::puzzle::{Puzzle, PuzzleMove, Transformable};
use crate::solver::moveset::TransitionTable;
use crate::solver::solution::Solution;

use crate::steps;
use crate::steps::step::{DefaultStepOptions, Step};

pub mod lookup_table;
pub mod stream;
pub mod solution;
pub mod df_search;
pub mod moveset;
pub use tokio_util::sync::CancellationToken;

pub fn solve_steps<'a, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>>(puzzle: PuzzleParam, steps: &'a Vec<(Step<'a, Turn, Transformation, PuzzleParam, TransTable>, DefaultStepOptions)>, cancel_token: CancellationToken) -> impl Iterator<Item = Solution<Turn>> + 'a {
    let first_step: Box<dyn Iterator<Item = Solution<Turn>>> = Box::new(vec![Solution::new()].into_iter());

    let solutions: Box<dyn Iterator<Item=Solution<Turn>>> = steps.iter()
        .fold(first_step, |acc, (step, search_opts)|{
            debug!("Step {} with options {:?}", step.kind(), search_opts);
            let next = steps::step::next_step(acc, step, search_opts.clone(), puzzle.clone(), cancel_token.clone())
                .zip(0..)
                .take_while(|(_, count)| search_opts.step_limit.map(|limit| limit > *count).unwrap_or(true))
                .map(|(sol, _)|sol);
            Box::new(next)
        });

    solutions
}

pub struct SolutionIterator<'a, Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>> {
    #[allow(unused)]
    steps: Vec<(Step<'a, Turn, Transformation, PuzzleParam, TransTable>, DefaultStepOptions)>,
    solutions: Box<dyn Iterator<Item=Solution<Turn>>>,
}

impl <Turn: PuzzleMove + Transformable<Transformation>, Transformation: PuzzleMove, PuzzleParam: Puzzle<Turn, Transformation>, TransTable: TransitionTable<Turn>> Iterator for SolutionIterator<'_, Turn, Transformation, PuzzleParam, TransTable> {
    type Item = Solution<Turn>;

    fn next(&mut self) -> Option<Self::Item> {
        self.solutions.next()
    }
}
