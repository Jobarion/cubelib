use std::fmt::{Debug, Display, Formatter};
use std::ptr::write;
use std::str::FromStr;
use crate::cube::Move;

pub struct Algorithm {
    pub normal_moves: Vec<Move>,
    pub inverse_moves: Vec<Move>,
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

impl Algorithm {
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