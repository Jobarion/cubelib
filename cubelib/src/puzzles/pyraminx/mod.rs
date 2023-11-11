use crate::puzzles::pyraminx::Direction::*;
use crate::puzzles::pyraminx::PyraminxTip::*;
use crate::puzzles::puzzle::{Invertible, PuzzleMove};

mod pyraminx;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PyraminxTurn {
    pub tip: PyraminxTip,
    pub dir: Direction,
}
pub type PyraminxTrans = PyraminxTurn;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Clockwise = 0,
    CounterClockwise = 1
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum PyraminxTip {
    Up,
    Left,
    Right,
    Back
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
        PyraminxTurn::new(self.tip, self.dir.invert())
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

impl PuzzleMove for PyraminxTurn {

}

#[allow(non_upper_case_globals)]
impl PyraminxTurn {
    pub const U: PyraminxTurn = PyraminxTurn::new(Up, Clockwise);
    pub const Ui: PyraminxTurn = PyraminxTurn::new(Up, CounterClockwise);
    pub const L: PyraminxTurn = PyraminxTurn::new(Left, Clockwise);
    pub const Li: PyraminxTurn = PyraminxTurn::new(Left, CounterClockwise);
    pub const R: PyraminxTurn = PyraminxTurn::new(Right, Clockwise);
    pub const Ri: PyraminxTurn = PyraminxTurn::new(Right, CounterClockwise);
    pub const B: PyraminxTurn = PyraminxTurn::new(Back, Clockwise);
    pub const Bi: PyraminxTurn = PyraminxTurn::new(Back, CounterClockwise);

    pub const ALL: [PyraminxTurn; 8] = [Self::U, Self::Ui, Self::L, Self::Li, Self::R, Self::Ri, Self::B, Self::Bi];

    pub const fn new(tip: PyraminxTip, dir: Direction) -> Self {
        PyraminxTurn { tip, dir }
    }

    pub const fn to_id(&self) -> usize {
        self.tip as usize * 2 + self.dir as usize
    }
}
