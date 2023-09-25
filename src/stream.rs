use std::collections::HashSet;
use crate::algs::{Algorithm, Solution};

pub(crate) fn iterated_dfs<
    'a,
    IN: Iterator<Item = Solution> + 'a,
    OUT: Iterator<Item = Solution> + 'a,
    F: 'a,
>(
    current_stage: IN,
    mapper: F,
) -> impl Iterator<Item = Solution> + 'a
where
    F: Fn(Solution, u8) -> OUT,
{
    DFSSolutionIter::new(current_stage)
        .take_while(|(_, depth)| *depth < 100)
        .flat_map(move |(alg, depth)| {
            let next_stage_depth = depth - alg.len();
            mapper(alg, next_stage_depth as u8)
        })
}

pub struct DFSSolutionIter<I> {
    orig: I,
    pos: usize,
    cycle_count: usize,
    cached_values: Vec<Solution>,
}

impl<I> DFSSolutionIter<I>
where
    I: Iterator<Item = Solution>,
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

impl<I> Iterator for DFSSolutionIter<I>
where
    I: Iterator<Item = Solution>,
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

struct DistinctSolutions<I> {
    orig: I,
    observed: HashSet<Algorithm>,
    current_length: usize,
}


impl<I> DistinctSolutions<I>
    where
        I: Iterator<Item = Solution>,
{
    fn new(iter: I) -> Self {
        Self {
            orig: iter,
            current_length: 0,
            observed: HashSet::new()
        }
    }
}

impl<I> Iterator for DistinctSolutions<I>
    where
        I: Iterator<Item = Solution>,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.orig.next() {
            None => None,
            Some(v) => {
                let alg: Algorithm = v.clone().into();
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

pub fn distinct_solutions(iter: impl Iterator<Item = Solution>) -> impl Iterator<Item = Solution> {
    DistinctSolutions::new(iter)
}