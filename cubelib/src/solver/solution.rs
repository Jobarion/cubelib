use std::fmt::{Debug, Display, Formatter};

use crate::algs::Algorithm;
use crate::cube::turn::{Invertible, InvertibleMut, TurnableMut, CubeOuterTurn, Direction};
use crate::defs::StepKind;

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq)]
pub struct Solution {
    pub steps: Vec<SolutionStep>,
    pub ends_on_normal: bool,
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct SolutionStep {
    pub kind: StepKind,
    pub variant: String,
    pub alg: Algorithm,
    pub comment: String
}

impl SolutionStep {

    // True if this step represents a class of steps
    // where the moves along the final axis may be inverted
    pub fn is_canonical(&self) -> bool {
        if self.kind == StepKind::FIN {
            return true;
        }
        fn is_canonical(vec: &Vec<CubeOuterTurn>) -> bool {
            match vec.len() {
                0 => true,
                1 => vec[0].dir != Direction::CounterClockwise,
                n =>
                    vec[n - 1].dir != Direction::CounterClockwise &&
                        (
                            vec[n - 2].face != vec[n - 1].face.opposite() ||
                                vec[n - 2].dir != Direction::CounterClockwise
                        ),
            }
        }
        is_canonical(&self.alg.normal_moves) && is_canonical(&self.alg.inverse_moves)
    }

    // Return all equivalent steps from reversing the last move
    pub fn equivalents(&self) -> Box<dyn Iterator<Item = SolutionStep> +'_> {
        if self.is_canonical() {
            return Box::new(vec![self.clone()].into_iter());
        }
        fn expand(moves: &Vec<CubeOuterTurn>) -> Vec<Vec<CubeOuterTurn>> {
            let n = moves.len();
            if n > 1 && moves[n - 1].face.opposite() == moves[n - 2].face {
                // Reverse last move
                let mut v1 = moves.clone();
                v1[n-1..n].iter_mut().for_each(|m| *m = m.invert());
                // Reverse penultimate move
                let mut v2 = moves.clone();
                v2[n-2..n-1].iter_mut().for_each(|m| *m = m.invert());
                // Reverse both
                let mut v3 = moves.clone();
                v3[n-2..n].iter_mut().for_each(|m| *m = m.invert());
                vec![moves.clone(), v1, v2, v3]
            } else if n > 0 {
                // Reverse last move
                let mut v = moves.clone();
                v[n-1..n].iter_mut().for_each(|m| *m = m.invert());
                vec![moves.clone(), v]
            }
            else {
                vec![moves.clone()]
            }
        }
        let mut algs = vec![];
        for n in expand(&self.alg.normal_moves) {
            for i in expand(&self.alg.inverse_moves) {
                algs.push(Algorithm { normal_moves: n.clone(), inverse_moves: i.clone() });
            }
        }
        Box::new(algs.into_iter().map(|alg| SolutionStep {
            kind: self.kind.clone(),
            variant: self.variant.clone(),
            alg,
            comment: self.comment.clone()
        }))
    }

}

impl Solution {
    pub fn new() -> Solution {
        Solution { steps: vec![], ends_on_normal: true }
    }

    pub fn len(&self) -> usize {
        self.steps.iter().map(|e| e.alg.len()).sum::<usize>()
    }

    pub fn add_step(&mut self, step: SolutionStep) {
        self.ends_on_normal = match (step.alg.normal_moves.is_empty(), step.alg.inverse_moves.is_empty()) {
            (true, true) => self.ends_on_normal,
            (false, false) => !self.ends_on_normal,
            (true, false) => false,
            (false, true) => true,
        };
        self.steps.push(step);
    }

    pub fn ends_on_normal(&self) -> bool {
        self.ends_on_normal
    }

    pub fn get_steps(&self) -> &'_ Vec<SolutionStep> {
        &self.steps
    }

    pub fn is_canonical(&self) -> bool {
        self.steps.iter().all(SolutionStep::is_canonical)
    }

    // Return all equivalent solutions
    pub fn equivalents(&self) -> Box<dyn Iterator<Item = Solution> +'_> {
        if self.steps.is_empty() {
            return Box::new(vec![self.clone()].into_iter());
        }
        Box::new(self.steps.last().unwrap().equivalents().map(|step| Solution {
            steps: self.steps[..self.steps.len() - 1].to_vec().into_iter().chain(vec![step]).collect(),
            ends_on_normal: self.ends_on_normal,
        }))
    }


    pub fn compact(self) -> Self {
        let mut steps: Vec<SolutionStep> = vec![];

        let mut i = 0;
        while i < self.steps.len() {
            let mut step = self.steps.get(i).cloned().unwrap();
            while i < self.steps.len() - 1 {
                let next = self.steps.get(i + 1).unwrap();
                if next.alg.len() == 0 {
                    // name.push_str(", ");
                    // name.push_str(next.0.as_str());
                    step.variant = next.variant.clone();
                    step.kind = next.kind.clone();
                } else {
                    break;
                }
                i += 1;
            }
            steps.push(step);
            i += 1;
        }
        Solution {
            steps,
            ends_on_normal: self.ends_on_normal
        }
    }
}

impl Into<Algorithm> for Solution {
    fn into(self) -> Algorithm {
        let mut start = Algorithm::new();
        for step in self.steps {
            start = start + step.alg;
        }
        start
    }
}

impl Clone for Solution {
    fn clone(&self) -> Self {
        Solution {
            steps: self.steps.clone(),
            ends_on_normal: self.ends_on_normal
        }
    }
}

impl Debug for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for n in 0..self.steps.len() {
            let step = self.steps.get(n).unwrap();
            write!(f, "{}-{}: {}", step.kind, step.variant, step.alg)?;
            if n < self.steps.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let compact = self.clone().compact();
        let mut total_moves = 0;
        let longest_alg_length = compact
            .steps
            .iter()
            .map(|s| s.alg.to_string().len())
            .max()
            .unwrap_or(0);
        let longest_name_length = compact
            .steps
            .iter()
            .map(|s| s.kind.to_string().len() + if s.comment.is_empty() { 0 } else { s.comment.len() + 3 })
            .max()
            .unwrap_or(0);

        for step in compact.steps {
            let alg_length = step.alg.len();
            let name = if step.variant.is_empty() || step.kind == StepKind::FIN {
                step.kind.to_string()
            } else {
                let comment = if step.comment.is_empty() {
                    "".to_string()
                } else {
                    format!(" [{}]", step.comment)
                };
                format!("{}{comment}", step.kind.to_string())
            };
            total_moves += alg_length;
            writeln!(f, "{:longest_alg_length$}  // {name:longest_name_length$} ({alg_length}/{total_moves})", step.alg.to_string())?;
        }
        let final_alg: Algorithm = if self.steps.last().map(|x| x.kind == StepKind::FIN).unwrap_or(false) {
            Into::<Algorithm>::into(self.clone()).to_uninverted()
        } else {
            self.clone().into()
        };
        writeln!(
            f,
            "Solution ({}): {}",
            total_moves,
            final_alg
        )
    }
}

pub trait ApplySolution<C: TurnableMut> {
    fn apply_solution(&mut self, solution: &Solution);
}

impl <C: TurnableMut + InvertibleMut> ApplySolution<C> for C {
    fn apply_solution(&mut self, solution: &Solution) {
        for step in solution.steps.iter() {
            for m in step.alg.normal_moves.iter() {
                self.turn(m.clone());
            }
        }
        self.invert();
        for step in solution.steps.iter() {
            for m in step.alg.inverse_moves.iter() {
                self.turn(m.clone());
            }
        }
        self.invert();
    }
}