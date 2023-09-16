use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::ptr::write;
use std::str::FromStr;
use itertools::Itertools;
use crate::cube::Move;
use crate::{Transformation, Turnable};

#[derive(PartialEq, Eq, Hash)]
pub struct Algorithm {
    pub normal_moves: Vec<Move>,
    pub inverse_moves: Vec<Move>,
}

impl Clone for Algorithm {
    fn clone(&self) -> Self {
        Algorithm {
            normal_moves: self.normal_moves.clone(),
            inverse_moves: self.inverse_moves.clone()
        }
    }
}

impl Display for Algorithm {
    fn fmt(&self, mut f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.normal_moves.len(), self.inverse_moves.len()) {
            (_, 0) => write!(f, "{}", Algorithm::fmt_alg(&self.normal_moves)),
            (0, _) => write!(f, "({})", Algorithm::fmt_alg(&self.inverse_moves)),
            _ => write!(f, "{} ({})", Algorithm::fmt_alg(&self.normal_moves), Algorithm::fmt_alg(&self.inverse_moves)),
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
            inverse_moves: vec![]
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

impl Turnable for Algorithm {
    fn turn(&mut self, m: Move) {
        self.normal_moves.push(m);
    }

    fn transform(&mut self, t: Transformation) {
        self.normal_moves = self.normal_moves.iter()
            .map(|m|m.transform(t))
            .collect_vec();
        self.inverse_moves = self.inverse_moves.iter()
            .map(|m|m.transform(t))
            .collect_vec();
    }
}

pub fn parse_algorithm(moves: &str) -> Vec<Move> {
    let mut chars = moves.chars();
    let mut moves: Vec<Move> = vec![];
    let mut current = "".to_string();
    for c in chars.filter(|c|!c.is_whitespace()) {
        current.push(c);
        if let Err(_) = Move::from_str(current.as_str()) {
            let mut chars = current.chars();
            chars.next_back();
            let previous = chars.as_str();
            moves.push(Move::from_str(previous).unwrap());
            current = String::from(c);
        }
    }
    moves.push(Move::from_str(current.as_str()).unwrap());
    moves
}