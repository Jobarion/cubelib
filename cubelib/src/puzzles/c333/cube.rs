use std::fmt::{Display, Formatter};

use crate::puzzles::c333::{CornerCube333, EdgeCube333, Transformation333, Turn333};
#[cfg(target_feature = "avx2")]
use crate::puzzles::c333::steps::eo::eo_config::EOCount;
use crate::puzzles::cube::{CornerPosition, EdgePosition};
use crate::puzzles::cube::{CubeColor, CubeOuterTurn};
use crate::puzzles::cube::CornerPosition::*;
use crate::puzzles::cube::CubeColor::*;
use crate::puzzles::cube::CubeFace::*;
use crate::puzzles::cube::EdgePosition::*;
use crate::puzzles::puzzle::{InvertibleMut, TransformableMut, TurnableMut};

//http://kociemba.org/math/cubielevel.htm
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(target_feature = "avx2", derive(Hash))]
pub struct Cube333 {
    pub edges: EdgeCube333,
    pub corners: CornerCube333,
}

impl Into<EdgeCube333> for &Cube333 {
    fn into(self) -> EdgeCube333 {
        self.edges
    }
}

impl Into<CornerCube333> for &Cube333 {
    fn into(self) -> CornerCube333 {
        self.corners
    }
}

impl Cube333 {
    pub fn new(edges: EdgeCube333, corners: CornerCube333) -> Cube333 {
        Cube333 { edges, corners }
    }

    #[cfg(target_feature = "avx2")]
    pub fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

impl Display for Cube333 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let facelets = self.get_facelets();
        let block_width = "   ";
        let block_spacing = " ";
        for x in (0..3).rev() {
            write!(f, "{}{}", block_width.repeat(3), block_spacing.repeat(4))?;
            for y in (0..3).rev() {
                write!(f, "{}{}", facelets[Back][x * 3 + y], block_spacing)?;
            }
            writeln!(f)?;
        }
        for x in 0..3 {
            let x_rev = 2 - x;
            for y in (0..3).rev() {
                write!(f, "{}{}", facelets[Left][x + y * 3], block_spacing)?;
            }
            write!(f, "{}", block_spacing)?;
            for y in 0..3 {
                write!(f, "{}{}", facelets[Up][x * 3 + y], block_spacing)?;
            }
            write!(f, "{}", block_spacing)?;
            for y in 0..3 {
                write!(
                    f,
                    "{}{}",
                    facelets[Right][x_rev + y * 3],
                    block_spacing
                )?;
            }
            write!(f, "{}", block_spacing)?;
            for y in (0..3).rev() {
                write!(
                    f,
                    "{}{}",
                    facelets[Down][x_rev * 3 + y],
                    block_spacing
                )?;
            }
            writeln!(f)?;
        }

        for x in 0..3 {
            write!(f, "{}{}", block_width.repeat(3), block_spacing.repeat(4))?;
            for y in 0..3 {
                write!(f, "{}{}", facelets[Front][x * 3 + y], block_spacing)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TurnableMut<CubeOuterTurn> for Cube333 {
    fn turn(&mut self, m: Turn333) {
        self.edges.turn(m);
        self.corners.turn(m);
    }
}

impl TransformableMut<Transformation333> for Cube333 {
    fn transform(&mut self, t: Transformation333) {
        self.edges.transform(t);
        self.corners.transform(t);
    }
}

impl InvertibleMut for Cube333 {
    #[inline]
    fn invert(&mut self) {
        self.edges.invert();
        self.corners.invert();
    }
}

impl Default for Cube333 {
    #[inline]
    fn default() -> Cube333 {
        Cube333 {
            edges: EdgeCube333::default(),
            corners: CornerCube333::default(),
        }
    }
}

impl Cube333 {
    pub const CORNER_COLORS: [[CubeColor; 3]; 8] = [
        [White, Orange, Blue],
        [White, Blue, Red],
        [White, Red, Green],
        [White, Green, Orange],
        [Yellow, Orange, Green],
        [Yellow, Green, Red],
        [Yellow, Red, Blue],
        [Yellow, Blue, Orange],
    ];

    const EDGE_COLORS: [[CubeColor; 2]; 12] = [
        [White, Blue],
        [White, Red],
        [White, Green],
        [White, Orange],
        [Green, Red],
        [Green, Orange],
        [Blue, Red],
        [Blue, Orange],
        [Yellow, Green],
        [Yellow, Red],
        [Yellow, Blue],
        [Yellow, Orange],
    ];

    pub fn get_facelets(&self) -> [[CubeColor; 9]; 6] {
        let corners = self.corners.get_corners();
        let edges = self.edges.get_edges();
        let mut facelets = [[None; 9]; 6];

        //There has to be a better way
        let c = |id: CornerPosition, twist: u8| {
            let corner = corners[id as usize];
            let twist_id = (3 - corner.orientation + twist) % 3;
            Cube333::CORNER_COLORS[corner.id as usize][twist_id as usize]
        };

        let e = |id: EdgePosition, flip: bool| {
            let edge = edges[id as usize];
            let eo_id = !(edge.oriented_fb ^ flip) as usize;
            Cube333::EDGE_COLORS[edge.id as usize][eo_id as usize]
        };

        facelets[Up][0] = c(UBL, 0);
        facelets[Up][1] = e(UB, false);
        facelets[Up][2] = c(UBR, 0);
        facelets[Up][3] = e(UL, false);
        facelets[Up][4] = White;
        facelets[Up][5] = e(UR, false);
        facelets[Up][6] = c(UFL, 0);
        facelets[Up][7] = e(UF, false);
        facelets[Up][8] = c(UFR, 0);

        facelets[Down][0] = c(DFL, 0);
        facelets[Down][1] = e(DF, false);
        facelets[Down][2] = c(DFR, 0);
        facelets[Down][3] = e(DL, false);
        facelets[Down][4] = Yellow;
        facelets[Down][5] = e(DR, false);
        facelets[Down][6] = c(DBL, 0);
        facelets[Down][7] = e(DB, false);
        facelets[Down][8] = c(DBR, 0);

        facelets[Front][0] = c(UFL, 1);
        facelets[Front][1] = e(UF, true);
        facelets[Front][2] = c(UFR, 2);
        facelets[Front][3] = e(FL, false);
        facelets[Front][4] = Green;
        facelets[Front][5] = e(FR, false);
        facelets[Front][6] = c(DFL, 2);
        facelets[Front][7] = e(DF, true);
        facelets[Front][8] = c(DFR, 1);

        facelets[Back][0] = c(UBR, 1);
        facelets[Back][1] = e(UB, true);
        facelets[Back][2] = c(UBL, 2);
        facelets[Back][3] = e(BR, false);
        facelets[Back][4] = Blue;
        facelets[Back][5] = e(BL, false);
        facelets[Back][6] = c(DBR, 2);
        facelets[Back][7] = e(DB, true);
        facelets[Back][8] = c(DBL, 1);

        facelets[Left][0] = c(UBL, 1);
        facelets[Left][1] = e(UL, true);
        facelets[Left][2] = c(UFL, 2);
        facelets[Left][3] = e(BL, true);
        facelets[Left][4] = Orange;
        facelets[Left][5] = e(FL, true);
        facelets[Left][6] = c(DBL, 2);
        facelets[Left][7] = e(DL, true);
        facelets[Left][8] = c(DFL, 1);

        facelets[Right][0] = c(UFR, 1);
        facelets[Right][1] = e(UR, true);
        facelets[Right][2] = c(UBR, 2);
        facelets[Right][3] = e(FR, true);
        facelets[Right][4] = Red;
        facelets[Right][5] = e(BR, true);
        facelets[Right][6] = c(DFR, 2);
        facelets[Right][7] = e(DR, true);
        facelets[Right][8] = c(DBR, 1);

        facelets
    }
}
