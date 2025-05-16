use std::fmt::{Debug, Display, Formatter};

use crate::algs::Algorithm;
use crate::cube::turn::{InvertibleMut, TurnableMut};
use crate::defs::{StepKind, StepVariant};

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq)]
pub struct Solution {
    pub steps: Vec<SolutionStep>,
    pub ends_on_normal: bool,
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct SolutionStep {
    pub variant: StepVariant,
    pub alg: Algorithm,
    pub comment: String
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

    pub fn compact(self) -> Self {
        let mut steps: Vec<SolutionStep> = vec![];

        let mut i = 0;
        while i < self.steps.len() {
            let mut step = self.steps.get(i).cloned().unwrap();
            while i < self.steps.len() - 1 {
                let next = self.steps.get(i + 1).unwrap();
                if next.alg.len() == 0 {
                    step.variant = next.variant.clone();
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
            write!(f, "{}: {}", step.variant, step.alg)?;
            if n < self.steps.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // let compact = self.clone().compact();
        let compact = self.clone();
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
            .map(|s| s.variant.to_string().len() + if s.comment.is_empty() { 0 } else { s.comment.len() + 3 })
            .max()
            .unwrap_or(0);

        for step in compact.steps {
            let alg_length = step.alg.len();

            let comment = if step.comment.is_empty() {
                "".to_string()
            } else {
                format!(" [{}]", step.comment)
            };
            let name = format!("{}{comment}", step.variant.to_string());
            total_moves += alg_length;
            writeln!(f, "{:longest_alg_length$}  // {name:longest_name_length$} ({alg_length}/{total_moves})", step.alg.to_string())?;
        }
        let final_alg: Algorithm = if self.steps.last().map(|x| StepKind::from(x.variant) == StepKind::FIN).unwrap_or(false) {
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