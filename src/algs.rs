use std::fmt::{Debug, Display, Formatter};
use std::fs::soft_link;
use std::ops::Add;
use std::ptr::write;
use std::str::FromStr;

use itertools::Itertools;

use crate::cube::{Move, Transformation, Turnable};

#[derive(PartialEq, Eq, Hash)]
pub struct Algorithm {
    pub normal_moves: Vec<Move>,
    pub inverse_moves: Vec<Move>,
}

impl Clone for Algorithm {
    fn clone(&self) -> Self {
        Algorithm {
            normal_moves: self.normal_moves.clone(),
            inverse_moves: self.inverse_moves.clone(),
        }
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.normal_moves.len(), self.inverse_moves.len()) {
            (_, 0) => write!(f, "{}", Algorithm::fmt_alg(&self.normal_moves)),
            (0, _) => write!(f, "({})", Algorithm::fmt_alg(&self.inverse_moves)),
            _ => write!(
                f,
                "{} ({})",
                Algorithm::fmt_alg(&self.normal_moves),
                Algorithm::fmt_alg(&self.inverse_moves)
            ),
        }
    }
}

impl Debug for Algorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Add for Algorithm {
    type Output = Algorithm;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        self.normal_moves.append(&mut rhs.normal_moves);
        self.inverse_moves.append(&mut rhs.inverse_moves);
        self
    }
}

impl Algorithm {
    pub fn new() -> Self {
        Algorithm {
            normal_moves: vec![],
            inverse_moves: vec![],
        }
    }

    pub fn reverse(mut self) -> Self {
        self.normal_moves.reverse();
        self.inverse_moves.reverse();
        self
    }

    pub fn len(&self) -> usize {
        self.normal_moves.len() + self.inverse_moves.len()
    }

    fn fmt_alg(moves: &Vec<Move>) -> String {
        if moves.is_empty() {
            return String::new();
        }
        let mut alg_string = String::new();
        for m in &moves[0..moves.len() - 1] {
            alg_string.push_str(m.to_string().as_str());
            alg_string.push_str(" ");
        }
        alg_string.push_str(moves[moves.len() - 1].to_string().as_str());
        alg_string
    }
}

impl FromStr for Algorithm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars();
        let mut moves: Vec<Move> = vec![];
        let mut inverse_moves: Vec<Move> = vec![];
        let mut current = "".to_string();
        let mut inverse = false;
        for c in chars.filter(|c| !c.is_whitespace()) {
            current.push(c);
            if let Err(_) = Move::from_str(current.as_str()) {
                let mut chars = current.chars();
                chars.next_back();
                let previous = chars.as_str();
                if !previous.is_empty() {
                    moves.push(Move::from_str(previous)?);
                }
                if c == '(' {
                    if inverse {
                        return Err(());
                    }
                    inverse = true;
                    std::mem::swap(&mut moves, &mut inverse_moves);
                    current = "".to_string();
                } else if c == ')' {
                    if !inverse {
                        return Err(());
                    }
                    inverse = false;
                    std::mem::swap(&mut moves, &mut inverse_moves);
                    current = "".to_string();
                } else {
                    current = String::from(c);
                }
            }
        }
        if !current.is_empty() {
            moves.push(Move::from_str(current.as_str())?);
        }
        Ok(Algorithm {
            normal_moves: moves,
            inverse_moves,
        })
    }
}

impl Turnable for Algorithm {
    fn turn(&mut self, m: Move) {
        self.normal_moves.push(m);
    }

    fn transform(&mut self, t: Transformation) {
        self.normal_moves = self
            .normal_moves
            .iter()
            .map(|m| m.transform(t))
            .collect_vec();
        self.inverse_moves = self
            .inverse_moves
            .iter()
            .map(|m| m.transform(t))
            .collect_vec();
    }
}

pub struct Solution {
    steps: Vec<(String, Algorithm)>,
}

impl Solution {
    pub fn new() -> Solution {
        Solution { steps: vec![] }
    }

    pub fn len(&self) -> usize {
        self.steps.iter().map(|e| e.1.len()).sum::<usize>()
    }

    pub fn add_step(&mut self, name: String, alg: Algorithm) {
        self.steps.push((name, alg));
    }

    pub fn get_steps(&self) -> &'_ Vec<(String, Algorithm)> {
        &self.steps
    }
}

impl Into<Algorithm> for Solution {
    fn into(self) -> Algorithm {
        let mut start = Algorithm::new();
        for (_, alg) in self.steps {
            start = start + alg;
        }
        start
    }
}

impl Clone for Solution {
    fn clone(&self) -> Self {
        Solution {
            steps: self.steps.clone(),
        }
    }
}

impl Debug for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for n in 0..self.steps.len() {
            let (name, alg) = self.steps.get(n).unwrap();
            write!(f, "{name}: {alg}")?;
            if n < self.steps.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut total_moves = 0;
        let mut i = 0;
        let longest_alg_length = self
            .steps
            .iter()
            .map(|(_, alg)| alg.to_string().len())
            .max()
            .unwrap_or(0);
        let longest_name_length = self
            .steps
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0);

        while i < self.steps.len() {
            let (mut name, alg) = self.steps.get(i).cloned().unwrap();
            let alg_length = alg.len();
            total_moves += alg_length;
            while i < self.steps.len() - 1 {
                let next = self.steps.get(i + 1).unwrap();
                if next.1.len() == 0 {
                    name = next.0.clone();
                } else {
                    break;
                }
                i += 1;
            }
            writeln!(f, "{:longest_alg_length$}  //{name:longest_name_length$} ({alg_length}/{total_moves})", alg.to_string())?;
            i += 1;
        }
        writeln!(
            f,
            "\nSolution ({}): {}",
            total_moves,
            Into::<Algorithm>::into(self.clone())
        )
    }
}

// pub trait ApplySolution<C: Turnable> {
//     fn apply_solution(cube: &mut C, solution: &Solution);
// }
//
// impl <C: Turnable> ApplySolution<C> for C {
//     fn apply_solution(cube: &mut C, solution: &Solution) {
//         for step in solution.steps.iter() {
//             step
//         }
//     }
// }

mod test {
    use crate::algs::Algorithm;
    use std::str::FromStr;

    #[test]
    fn empty() {
        let alg = Algorithm::from_str("").unwrap();
        assert_eq!(0, alg.normal_moves.len());
        assert_eq!(0, alg.inverse_moves.len());
    }

    #[test]
    fn single() {
        let alg = Algorithm::from_str("R").unwrap();
        assert_eq!(1, alg.normal_moves.len());
        assert_eq!(0, alg.inverse_moves.len());
    }

    #[test]
    fn single_inverse() {
        let alg = Algorithm::from_str("(R)").unwrap();
        assert_eq!(0, alg.normal_moves.len());
        assert_eq!(1, alg.inverse_moves.len());
    }

    #[test]
    fn multi_switch() {
        let alg = Algorithm::from_str("F (B) F (B F) B").unwrap();
        assert_eq!(3, alg.normal_moves.len());
        assert_eq!(3, alg.inverse_moves.len());
    }

    #[test]
    fn invalid_move() {
        let alg = Algorithm::from_str("P");
        assert!(alg.is_err());
    }

    #[test]
    fn invalid_move_inverse() {
        let alg = Algorithm::from_str("(P)");
        assert!(alg.is_err());
    }
}
