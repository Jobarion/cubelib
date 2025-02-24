use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

use itertools::Itertools;
use crate::puzzles::c333::{Transformation333, Turn333};
use crate::puzzles::puzzle::{Invertible, Transformable, TransformableMut, TurnableMut};

#[derive(PartialEq, Eq, Hash)]
//This is a pretty bad serialization format. We can do better if Turn is FromStr, but right now that's not enforced.
//Implementing (De)Serialize only when Turn is FromStr breaks Solution and SolutionStep right now.
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct Algorithm {
    pub normal_moves: Vec<Turn333>,
    pub inverse_moves: Vec<Turn333>,
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

    pub fn to_uninverted(mut self) -> Self {
        self.inverse_moves.reverse();
        for i in 0..self.inverse_moves.len() {
            self.inverse_moves[i] = self.inverse_moves[i].invert();
        }
        self.normal_moves.append(&mut self.inverse_moves);
        self
    }

    pub fn reverse(mut self) -> Self {
        self.normal_moves.reverse();
        self.inverse_moves.reverse();
        self
    }

    pub fn len(&self) -> usize {
        self.normal_moves.len() + self.inverse_moves.len()
    }
}

#[cfg(feature = "333")]
impl Algorithm {
    pub fn mirror(&mut self, axis: crate::puzzles::cube::CubeAxis) {
        self.normal_moves = self
            .normal_moves
            .iter()
            .map(|m| m.mirror(axis))
            .collect_vec();
        self.inverse_moves = self
            .inverse_moves
            .iter()
            .map(|m| m.mirror(axis))
            .collect_vec();
    }
}

impl Algorithm {
    fn fmt_alg(moves: &Vec<Turn333>) -> String {
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
        let mut moves: Vec<Turn333> = vec![];
        let mut inverse_moves: Vec<Turn333> = vec![];
        let mut current = "".to_string();
        let mut inverse = false;
        for c in chars.filter(|c| !c.is_whitespace()) {
            current.push(c);
            if let Err(_) = Turn333::from_str(current.as_str()) {
                let mut chars = current.chars();
                chars.next_back();
                let previous = chars.as_str();
                if !previous.is_empty() {
                    moves.push(Turn333::from_str(previous)?);
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
            moves.push(Turn333::from_str(current.as_str())?);
        }
        Ok(Algorithm {
            normal_moves: moves,
            inverse_moves,
        })
    }
}

impl TurnableMut for Algorithm {
    fn turn(&mut self, m: Turn333) {
        self.normal_moves.push(m);
    }
}

impl TransformableMut for Algorithm {
    fn transform(&mut self, t: Transformation333) {
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

// #[cfg(feature = "serde_support")]
// mod serde_support {
//     use std::fmt::{Display, Formatter};
//     use std::marker::PhantomData;
//     use std::str::FromStr;
//
//     use serde::{Deserialize, Deserializer, Serialize, Serializer};
//     use serde::de::{Error, Visitor};
//
//     use crate::algs::Algorithm;
//     use crate::puzzles::puzzle::PuzzleMove;
//
//     impl <T: PuzzleMove + Display> Serialize for Algorithm<T> {
//         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//             serializer.serialize_str(self.to_string().as_str())
//         }
//     }
//
//     struct AlgVisitor<T: PuzzleMove>(PhantomData<T>);
//
//     impl<'de, T: PuzzleMove + FromStr<Err = ()>> Visitor<'de> for AlgVisitor<T> {
//         type Value = Algorithm<T>;
//
//         fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
//             formatter.write_str("An algorithm in string format")
//         }
//
//         fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
//             Algorithm::<T>::from_str(v).map_err(|_| E::custom("Failed to parse algorithm string"))
//         }
//     }
//
//     impl<'de, T: PuzzleMove + FromStr<Err = ()>> Deserialize<'de> for Algorithm<T> {
//         fn deserialize<D>(deserializer: D) -> Result<Algorithm<T>, D::Error>
//             where
//                 D: Deserializer<'de>,
//         {
//             deserializer.deserialize_str(AlgVisitor(PhantomData::default()))
//         }
//     }
// }