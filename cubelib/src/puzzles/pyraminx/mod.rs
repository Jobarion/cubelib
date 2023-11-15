use std::fmt::{Display, Formatter, write};
use std::str::FromStr;
use crate::puzzles::pyraminx::Direction::*;
use crate::puzzles::pyraminx::PyraminxTip::*;
use crate::puzzles::puzzle::{Invertible, PuzzleMove, Transformable};

pub mod coords;
pub mod steps;
mod pyraminx;

pub use pyraminx::Pyraminx;
use crate::algs::Algorithm;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PyraminxTurn {
    pub tip: PyraminxTip,
    pub dir: Direction,
    pub tip_only: bool,
}
pub type PyraminxTransformation = PyraminxTurn;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Clockwise = 0,
    CounterClockwise = 1
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PyraminxTip {
    Up = 0,
    Left = 1,
    Right = 2,
    Back = 3
}

impl TryFrom<char> for PyraminxTip {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'U' => Ok(Up),
            'L' => Ok(Left),
            'R' => Ok(Right),
            'B' => Ok(Back),
            _ => Err(())
        }
    }
}

impl Into<char> for PyraminxTip {
    fn into(self) -> char {
        match self {
            Up => 'U',
            Left => 'L',
            Right => 'R',
            Back => 'B',
        }
    }
}

impl FromStr for PyraminxTurn {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let first = chars.next().ok_or(())?;
        let tip = PyraminxTip::try_from(first)?;
        let direction = match chars.next() {
            None => Ok(Clockwise),
            Some('\'') => Ok(CounterClockwise),
            _ => Err(())
        }?;
        let tip_only = first.is_ascii_lowercase();
        if chars.next().is_some() {
            Err(())
        } else {
            Ok(PyraminxTurn::new(tip, direction, tip_only))
        }
    }
}

impl Algorithm<PyraminxTurn> {
    pub fn split_tips_and_no_tips(&self) -> (Algorithm<PyraminxTurn>, Algorithm<PyraminxTurn>) {
        let mut no_tips = Algorithm::new();
        let mut tips = Algorithm::new();
        for normal in self.normal_moves.iter().cloned() {
            if normal.tip_only {
                tips.normal_moves.push(normal);
            } else {
                no_tips.normal_moves.push(normal);
            }
        }
        for inverse in self.inverse_moves.iter().cloned() {
            if inverse.tip_only {
                tips.inverse_moves.push(inverse);
            } else {
                no_tips.inverse_moves.push(inverse);
            }
        }
        (tips, no_tips)
    }
}

impl Display for PyraminxTurn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut tip: char = self.tip.into();
        if self.tip_only {
            tip = tip.to_ascii_lowercase();
        }
        write!(f, "{tip}")?;
        if self.dir == CounterClockwise {
            write!(f, "'")?;
        }
        Ok(())
    }
}

impl From<usize> for PyraminxTurn {
    fn from(value: usize) -> Self {
        Self::ALL[value]
    }
}

impl Into<usize> for PyraminxTurn {
    fn into(self) -> usize {
        self.to_id()
    }
}

impl Invertible for PyraminxTurn {
    fn invert(&self) -> Self {
        PyraminxTurn::new(self.tip, self.dir.invert(), self.tip_only)
    }
}

impl Invertible for Direction {
    fn invert(&self) -> Self {
        match self {
            Clockwise => CounterClockwise,
            CounterClockwise => Clockwise,
        }
    }
}

impl Transformable<PyraminxTransformation> for PyraminxTurn {
    fn transform(&self, transformation: PyraminxTransformation) -> Self {
        todo!()
    }
}

impl PuzzleMove for PyraminxTurn {
    fn all() -> &'static [Self] {
        &Self::ALL
    }
}

#[allow(non_upper_case_globals)]
impl PyraminxTurn {
    pub const U: PyraminxTurn = PyraminxTurn::new(Up, Clockwise, false);
    pub const Ui: PyraminxTurn = PyraminxTurn::new(Up, CounterClockwise, false);
    pub const L: PyraminxTurn = PyraminxTurn::new(Left, Clockwise, false);
    pub const Li: PyraminxTurn = PyraminxTurn::new(Left, CounterClockwise, false);
    pub const R: PyraminxTurn = PyraminxTurn::new(Right, Clockwise, false);
    pub const Ri: PyraminxTurn = PyraminxTurn::new(Right, CounterClockwise, false);
    pub const B: PyraminxTurn = PyraminxTurn::new(Back, Clockwise, false);
    pub const Bi: PyraminxTurn = PyraminxTurn::new(Back, CounterClockwise, false);
    pub const u: PyraminxTurn = PyraminxTurn::new(Up, Clockwise, true);
    pub const ui: PyraminxTurn = PyraminxTurn::new(Up, CounterClockwise, true);
    pub const l: PyraminxTurn = PyraminxTurn::new(Left, Clockwise, true);
    pub const li: PyraminxTurn = PyraminxTurn::new(Left, CounterClockwise, true);
    pub const r: PyraminxTurn = PyraminxTurn::new(Right, Clockwise, true);
    pub const ri: PyraminxTurn = PyraminxTurn::new(Right, CounterClockwise, true);
    pub const b: PyraminxTurn = PyraminxTurn::new(Back, Clockwise, true);
    pub const bi: PyraminxTurn = PyraminxTurn::new(Back, CounterClockwise, true);

    pub const ALL: [PyraminxTurn; 16] = [
        Self::u, Self::ui, Self::l, Self::li, Self::r, Self::ri, Self::b, Self::bi,
        Self::U, Self::Ui, Self::L, Self::Li, Self::R, Self::Ri, Self::B, Self::Bi,
    ];

    pub const NO_TIPS: [PyraminxTurn; 8] = [Self::U, Self::Ui, Self::L, Self::Li, Self::R, Self::Ri, Self::B, Self::Bi];
    pub const TIPS: [PyraminxTurn; 8] = [Self::u, Self::ui, Self::l, Self::li, Self::r, Self::ri, Self::b, Self::bi];

    pub const fn new(tip: PyraminxTip, dir: Direction, tip_only: bool) -> Self {
        PyraminxTurn { tip, dir, tip_only }
    }

    pub const fn to_id(&self) -> usize {
        // self.tip as usize * 2 * 2 + self.dir as usize * 2 + self.tip_only as usize
        ((self.tip as usize * 2 + self.dir as usize) << 1) + self.tip_only as usize
    }
}
