use crate::puzzles::cube::CubeCornersOdd;
use crate::steps::coord::Coord;
use crate::puzzles::cube::coords::EPCoord;
use crate::puzzles::cube::coords::COUDCoord;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CornerCoord(u64);

pub const CORNER_COORD_SIZE: usize = 40320 * 2187;
impl Coord<CORNER_COORD_SIZE> for CornerCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CornerCoord {
    fn into(self) -> usize {
        self.val()
    }
}

impl From<&CubeCornersOdd> for CornerCoord {
    fn from(value: &CubeCornersOdd) -> Self {
        let cp = EPCoord::from(value);
        let co = COUDCoord::from(value);
        let coord = co.val() + cp.val() * COUDCoord::size();
        Self(coord as u64)
    }
}
