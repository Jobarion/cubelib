use std::vec;
use log::debug;
use crate::cube::Cube333;

use crate::solver::solution::Solution;

use crate::steps;

pub mod lookup_table;
pub mod stream;
pub mod solution;
pub mod df_search;
pub mod moveset;
use crate::solver::df_search::CancelToken;
use crate::steps::step::{DefaultStepOptions, Step};

pub fn solve_steps<'a>(puzzle: Cube333, steps: &'a Vec<(Step<'a>, DefaultStepOptions)>, cancel_token: &'a CancelToken) -> Vec<Solution> {
    let mut solutions = vec![Solution::new()];

    for (step, search_opts) in steps {
        debug!("Step {} with options {:?}", step.kind(), search_opts);
        let mut next_step_solutions = vec![];
        for i in 0..solutions.len() {
            let iter = steps::step::next_step(
                solutions[i..i+1].iter().cloned(),
                step,
                search_opts.clone(),
                puzzle.clone(),
                cancel_token
            );
            if step.kind.can_reverse_final_move() {
                next_step_solutions.extend(iter.filter(Solution::is_canonical));
            }
            else {
                next_step_solutions.extend(iter);
            }
        }
        match search_opts.step_limit {
            Some(limit) => {
                debug!("Found {} {}'s. Selecting {}", next_step_solutions.len(), step.kind, limit);
                next_step_solutions.sort_by(|a, b| a.len().cmp(&b.len()));
                if step.kind.can_reverse_final_move() {
                    solutions = next_step_solutions[..limit]
                        .iter()
                        .flat_map(Solution::equivalents)
                        .collect();
                }
                else {
                    solutions = next_step_solutions[..limit].to_vec();
                }
            }
            None => {
                debug!("Found {} {}'s. Keeping all", next_step_solutions.len(), step.kind);
                if step.kind.can_reverse_final_move() {
                    solutions = next_step_solutions
                        .iter()
                        .flat_map(Solution::equivalents)
                        .collect();
                }
                else {
                    solutions = next_step_solutions;
                }
            }
        }
    }
    solutions
}

pub struct SolutionIterator<'a> {
    #[allow(unused)]
    steps: Vec<(Step<'a>, DefaultStepOptions)>,
    solutions: Box<dyn Iterator<Item=Solution>>,
}

impl Iterator for SolutionIterator<'_> {
    type Item = Solution;

    fn next(&mut self) -> Option<Self::Item> {
        self.solutions.next()
    }
}
