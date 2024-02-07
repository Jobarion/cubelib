use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use crate::puzzles::cube::{CubeAxis, CubeOuterTurn, CubeTransformation, Direction};
use crate::puzzles::cube::Direction::{Clockwise, CounterClockwise, Half};
use crate::puzzles::puzzle::{Invertible, PuzzleMove, Transformable};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub enum CubicOddOuterAndSliceTurn {
    Outer(CubeOuterTurn),
    Slice(CubeSliceTurn),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct CubeSliceTurn {
    pub axis: CubeAxis,
    pub dir: Direction,
}

impl Invertible for CubeSliceTurn {
    fn invert(&self) -> CubeSliceTurn {
        CubeSliceTurn {
            axis: self.axis,
            dir: self.dir.invert()
        }
    }
}

impl PuzzleMove for CubeSliceTurn {
    fn all() -> &'static [Self] {
        &Self::ALL
    }
}

impl Transformable<CubeTransformation> for CubeSliceTurn {
    fn transform(&self, transformation: CubeTransformation) -> Self {
        Self::new(self.axis.transform(transformation), self.dir)
    }
}

impl From<usize> for CubeSliceTurn {
    fn from(value: usize) -> Self {
        Self::ALL[value]
    }
}

impl Into<usize> for CubeSliceTurn {
    fn into(self) -> usize {
        self.to_id()
    }
}

#[allow(non_upper_case_globals)]
impl CubeSliceTurn {
    pub const E: Self = Self::new(CubeAxis::UD, Clockwise);
    pub const E2: Self = Self::new(CubeAxis::UD, Half);
    pub const Ei: Self = Self::new(CubeAxis::UD, CounterClockwise);
    pub const S: Self = Self::new(CubeAxis::FB, Clockwise);
    pub const S2: Self = Self::new(CubeAxis::FB, Half);
    pub const Si: Self = Self::new(CubeAxis::FB, CounterClockwise);
    pub const M: Self = Self::new(CubeAxis::LR, Clockwise);
    pub const M2: Self = Self::new(CubeAxis::LR, Half);
    pub const Mi: Self = Self::new(CubeAxis::LR, CounterClockwise);

    pub const ALL: [CubeSliceTurn; 9] = [
        Self::E, Self::Ei, Self::E2,
        Self::S, Self::Si, Self::S2,
        Self::M, Self::Mi, Self::M2,
    ];

    pub const fn new(axis: CubeAxis, dir: Direction) -> Self {
        Self { axis, dir }
    }

    pub fn mirror(&self, _: CubeAxis) -> Self {
        Self::new(self.axis, self.dir.invert())
    }

    pub const fn to_id(&self) -> usize {
        self.axis as usize * 3 + self.dir as usize
    }
}

impl Display for CubeSliceTurn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice = match self.axis {
            CubeAxis::UD => "E",
            CubeAxis::FB => "S",
            CubeAxis::LR => "M",
        }.to_string();
        let turn = match self.dir {
            Clockwise => "",
            CounterClockwise => "'",
            Half => "2",
        };
        write!(f, "{slice}{turn}")
    }
}

impl Debug for CubeSliceTurn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for CubeSliceTurn {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut chars = value.chars();
        let slice = match chars.next() {
            Some('E') => Ok(CubeAxis::UD),
            Some('S') => Ok(CubeAxis::FB),
            Some('M') => Ok(CubeAxis::LR),
            _ => Err(()),
        }?;
        let turn = match chars.next() {
            Some('2') => Ok(Direction::Half),
            Some('\'') => Ok(Direction::CounterClockwise),
            None => Ok(Direction::Clockwise),
            _ => Err(()),
        }?;
        if chars.next().is_none() {
            Ok(CubeSliceTurn::new(slice, turn))
        } else {
            Err(())
        }
    }
}
