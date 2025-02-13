use std::fmt::{Debug, Display, Formatter};

use crate::algs::Algorithm;
use crate::defs::StepKind;
use crate::puzzles::puzzle::{InvertibleMut, PuzzleMove, TurnableMut};

#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq)]
pub struct Solution<Turn: PuzzleMove> {
    pub steps: Vec<SolutionStep<Turn>>,
    pub ends_on_normal: bool,
}

#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct SolutionStep<Turn: PuzzleMove> {
    pub kind: StepKind,
    pub variant: String,
    pub alg: Algorithm<Turn>,
    pub comment: String
}

impl <Turn: PuzzleMove> Solution<Turn> {
    pub fn new() -> Solution<Turn> {
        Solution { steps: vec![], ends_on_normal: true }
    }

    pub fn len(&self) -> usize {
        self.steps.iter().map(|e| e.alg.len()).sum::<usize>()
    }

    pub fn add_step(&mut self, step: SolutionStep<Turn>) {
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

    pub fn get_steps(&self) -> &'_ Vec<SolutionStep<Turn>> {
        &self.steps
    }

    pub fn compact(self) -> Self {
        let mut steps: Vec<SolutionStep<Turn>> = vec![];

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

impl <Turn: PuzzleMove> Into<Algorithm<Turn>> for Solution<Turn> {
    fn into(self) -> Algorithm<Turn> {
        let mut start = Algorithm::new();
        for step in self.steps {
            start = start + step.alg;
        }
        start
    }
}

impl <Turn: PuzzleMove> Clone for Solution<Turn> {
    fn clone(&self) -> Self {
        Solution {
            steps: self.steps.clone(),
            ends_on_normal: self.ends_on_normal
        }
    }
}

impl <Turn: PuzzleMove + Display> Debug for Solution<Turn> {
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

impl <Turn: PuzzleMove + Display> Display for Solution<Turn> {
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
            .map(|s| s.kind.to_string().len() + if s.comment.is_empty() { 0 } else { s.comment.len() + 1 })
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
            writeln!(f, "{:longest_alg_length$}  //{name:longest_name_length$} ({alg_length}/{total_moves})", step.alg.to_string())?;
        }
        let final_alg: Algorithm<Turn> = if self.steps.last().map(|x| x.kind == StepKind::FIN).unwrap_or(false) {
            Into::<Algorithm<Turn>>::into(self.clone()).to_uninverted()
        } else {
            self.clone().into()
        };
        writeln!(
            f,
            "\nSolution ({}): {}",
            total_moves,
            final_alg
        )
    }
}

pub trait ApplySolution<Turn: PuzzleMove, C: TurnableMut<Turn>> {
    fn apply_solution(&mut self, solution: &Solution<Turn>);
}

impl <Turn: PuzzleMove, C: TurnableMut<Turn> + InvertibleMut> ApplySolution<Turn, C> for C {
    fn apply_solution(&mut self, solution: &Solution<Turn>) {
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