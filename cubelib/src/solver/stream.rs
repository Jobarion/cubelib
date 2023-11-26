use std::collections::HashSet;
use std::marker::PhantomData;

use tokio_util::sync::CancellationToken;

use crate::algs::Algorithm;
use crate::puzzles::puzzle::PuzzleMove;
use crate::solver::solution::Solution;

pub(crate) fn iterated_dfs<
    'a,
    Turn: PuzzleMove + 'a,
    IN: Iterator<Item = Solution<Turn>> + 'a,
    OUT: Iterator<Item = Solution<Turn>> + 'a,
    F: 'a,
>(
    current_stage: IN,
    cancel_token: CancellationToken,
    mapper: F,
) -> impl Iterator<Item = Solution<Turn>> + 'a
where
    F: Fn(Solution<Turn>, u8, CancellationToken) -> OUT,
{
    let ct1 = cancel_token.clone();
    DFSSolutionIter::new(current_stage)
        .take_while(move |_|!ct1.is_cancelled())
        .take_while(|(_, depth)| *depth < 100)
        .flat_map(move |(alg, depth)| {
            let next_stage_depth = depth - alg.len();
            mapper(alg, next_stage_depth as u8, cancel_token.clone())
        })
}

pub struct DFSSolutionIter<I, Turn: PuzzleMove> {
    orig: I,
    pos: usize,
    cycle_count: usize,
    cached_values: Vec<Solution<Turn>>,
}

impl<I, Turn: PuzzleMove> DFSSolutionIter<I, Turn>
where
    I: Iterator<Item = Solution<Turn>>,
{
    pub fn new(iter: I) -> Self {
        Self {
            orig: iter,
            pos: 0,
            cycle_count: 0,
            cached_values: vec![],
        }
    }
}

impl<I, Turn: PuzzleMove> Iterator for DFSSolutionIter<I, Turn>
where
    I: Iterator<Item = Solution<Turn>>,
{
    type Item = (<I as Iterator>::Item, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos {
            n if self.cached_values.len() == n => match self.orig.next() {
                None if self.cached_values.len() == 0 => None,
                None => {
                    self.pos = 0;
                    self.cycle_count += 1;
                    self.next()
                }
                Some(t) => {
                    self.cached_values.push(t);
                    self.next()
                }
            },
            n => {
                let alg = self.cached_values[n].clone();
                if alg.len() <= self.cycle_count {
                    self.pos += 1;
                    Some((alg, self.cycle_count))
                } else {
                    self.pos = 0;
                    self.cycle_count += 1;
                    self.next()
                }
            }
        }
    }
}

struct DistinctSolutions<I, V, Turn: PuzzleMove> {
    orig: I,
    observed: HashSet<Algorithm<Turn>>,
    current_length: usize,
    _v: PhantomData<V>,
}

impl<I, V, Turn: PuzzleMove> DistinctSolutions<I, V, Turn>
    where
        I: Iterator<Item = V>,
        V: Into<Algorithm<Turn>> + Clone
{
    fn new(iter: I) -> Self {
        Self {
            orig: iter,
            current_length: 0,
            observed: HashSet::new(),
            _v: PhantomData::default(),
        }
    }
}

impl<I, V, Turn: PuzzleMove> Iterator for DistinctSolutions<I, V, Turn>
    where
        I: Iterator<Item = V>,
        V: Into<Algorithm<Turn>> + Clone
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.orig.next() {
            None => None,
            Some(v) => {
                let alg: Algorithm<Turn> = v.clone().into();
                if alg.len() > self.current_length {
                    self.observed.clear();
                    self.current_length = alg.len();
                    self.observed.insert(alg);
                    Some(v)
                } else if self.observed.insert(alg) {
                    Some(v)
                } else {
                    self.next()
                }
            }
        }
    }
}

pub fn distinct_algorithms<Turn: PuzzleMove, V: Into<Algorithm<Turn>> + Clone>(iter: impl Iterator<Item = V>) -> impl Iterator<Item = V> {
    DistinctSolutions::<_, V, Turn>::new(iter)
}