use crate::algs::{Solution};

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

// pub fn merge_iters<'a, A: Iterator<Item=Algorithm> + 'a, B: Iterator<Item=Algorithm> + 'a>(a: A, b: B) -> impl Iterator<Item = Algorithm> + 'a {
//     PickingIter::new(a, b)
// }

// pub struct MultiMergeIter {
//     current_iter: usize,
//     current_min: usize,
//     iters: Vec<Peekable<Box<dyn Iterator<Item = Algorithm>>>>,
// }
//
// impl MultiMergeIter {
//     pub fn new(iters: impl Iterator<Item = impl Iterator<Item = Algorithm>>) -> Self {
//         let peeked = iters.into_iter()
//             .map(|i| Box::new(i))
//             .map(|i| i.peekable())
//             .collect_vec();
//         MultiMergeIter {
//             iters: peeked,
//             current_iter: 0,
//             current_min: 0
//         }
//     }
// }

// impl Iterator for MultiMergeIter {
//     type Item = Algorithm;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         while self.current_iter < self.iters.len() {
//             if let Some(alg) = self.iters[self.current_iter].peek() {
//                 if alg.len() <= self.current_min {
//                     return self.iters[self.current_iter].next()
//                 }
//             }
//             self.current_iter += 1;
//         }
//         self.current_min += 1;
//         self.current_iter = 0;
//         self.next()
//     }
// }
//
// pub struct PickingIter<A, B> {
//     a: A,
//     b: B,
//     a_val: Option<Algorithm>,
//     b_val: Option<Algorithm>,
// }
//
//
// impl<'a, A, B> PickingIter<A, B>
//     where
//         A: Iterator<Item = Algorithm>,
//         B: Iterator<Item = Algorithm>, {
//
//     pub fn new(mut a: A, mut b: B) -> Self {
//         PickingIter { a_val: a.next(), b_val: b.next(), a, b}
//     }
// }
//
// impl<'a, A, B> Iterator for PickingIter<A, B>
//     where
//         A: Iterator<Item = Algorithm>,
//         B: Iterator<Item = Algorithm>, {
//     type Item = Algorithm;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match (&self.a_val, &self.b_val) {
//             (None, None) => None,
//             (None, Some(_)) => mem::replace(&mut self.b_val, self.b.next()),
//             (Some(a), Some(b)) if a.len() > b.len() => mem::replace(&mut self.b_val, self.b.next()),
//             (Some(_), _) => mem::replace(&mut self.a_val, self.a.next()),
//         }
//     }
// }

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
