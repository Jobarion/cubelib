use crate::Algorithm;

pub struct StageOptions {
    niss_type: NissType,
}

pub enum NissType {
    None,
    AtStart,
    During
}

pub fn next_stage<'a, IN: Iterator<Item=Algorithm> + 'a, OUT: Iterator<Item=Algorithm> + 'a, F: 'a>(current_stage: IN, mapper: F) -> impl Iterator<Item = Algorithm> + 'a
    where F: Fn(Algorithm, u8) -> OUT {
    DFSAlgIter::new(current_stage)
        .flat_map(move |(alg, depth)| {
            let next_stage_depth = depth - alg.len();
            mapper(alg, next_stage_depth as u8)
        })
}

pub struct DFSAlgIter<I> {
    orig: I,
    pos: usize,
    cycle_count: usize,
    cached_values: Vec<Algorithm>,
}

impl<I> DFSAlgIter<I>
    where
        I: Iterator<Item = Algorithm> {

    pub fn new(iter: I) -> Self {
        Self {
            orig: iter,
            pos: 0,
            cycle_count: 0,
            cached_values: vec![]
        }
    }
}

impl<I> Iterator for DFSAlgIter<I>
    where
        I: Iterator<Item = Algorithm>,
{
    type Item = (<I as Iterator>::Item, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos {
            n if self.cached_values.len() == n => {
                match self.orig.next() {
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
                }
            }
            n=> {
                let alg = self.cached_values[n].clone();
                if alg.len() <= self.cycle_count {
                    self.pos += 1;
                    Some((alg, self.cycle_count))
                }
                else {
                    self.pos = 0;
                    self.cycle_count += 1;
                    self.next()
                }
            }
        }
    }
}