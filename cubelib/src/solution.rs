use std::fmt::{Debug, Display, Formatter};
use log::error;
use crate::algs::Algorithm;
use crate::cube::{Invertible, Turnable};
use crate::defs::StepKind;

pub struct Solution {
    pub steps: Vec<SolutionStep>,
    pub ends_on_normal: bool,
}

#[derive(Clone)]
pub struct SolutionStep {
    pub kind: StepKind,
    pub variant: String,
    pub alg: Algorithm,
}

impl Solution {
    pub fn new() -> Solution {
        Solution { steps: vec![], ends_on_normal: true }
    }

    pub fn len(&self) -> usize {
        self.steps.iter().map(|e| e.alg.len()).sum::<usize>()
    }

    pub fn add_step(&mut self, step: SolutionStep) {
        if step.alg.normal_moves.is_empty() {
            self.ends_on_normal = false;
        }
        else if step.alg.inverse_moves.is_empty() {
            self.ends_on_normal = true;
        } else {
            self.ends_on_normal = !self.ends_on_normal;
        }
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
                    // name.push_str(", ");
                    // name.push_str(next.0.as_str());
                    step.variant = next.variant.clone();
                    step.kind = next.kind;
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
            .map(|s| s.kind.to_string().len() + 1 + s.variant.to_string().len())
            .max()
            .unwrap_or(0);

        for step in compact.steps {
            let alg_length = step.alg.len();
            let name = if step.variant.is_empty() || step.kind == StepKind::FIN {
                step.kind.to_string()
            } else {
                format!("{}-{}", step.kind.to_string(), step.variant)
            };
            total_moves += alg_length;
            writeln!(f, "{:longest_alg_length$}  //{name:longest_name_length$} ({alg_length}/{total_moves})", step.alg.to_string())?;
        }
        writeln!(
            f,
            "\nSolution ({}): {}",
            total_moves,
            Into::<Algorithm>::into(self.clone())
        )
    }
}

pub trait ApplySolution<C: Turnable> {
    fn apply_solution(&mut self, solution: &Solution);
}

impl <C: Turnable + Invertible> ApplySolution<C> for C {
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