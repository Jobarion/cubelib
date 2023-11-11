use crate::steps::coord::Coord;
use crate::puzzles::c333::Cube333;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRUDFinishCoord(pub(crate) u8);

pub const FR_FINISH_SIZE: usize = 256;
impl Coord<{FR_FINISH_SIZE}> for FRUDFinishCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRUDFinishCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<&Cube333> for FRUDFinishCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &Cube333) -> Self {
        unsafe { avx2::unsafe_from_fr_finish_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &Cube333) -> Self {
        wasm32::from_fr_finish_coord(value)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_cmpeq_epi8, _mm_extract_epi16, _mm_movemask_epi8, _mm_or_si128, _mm_sad_epu8, _mm_set1_epi8, _mm_setr_epi8};

    use crate::puzzles::c333::Cube333;
    use crate::puzzles::c333::steps::finish::coords::FRUDFinishCoord;

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_finish_coord(cube: &Cube333) -> FRUDFinishCoord {
        let correct_ufl_corner_position = _mm_cmpeq_epi8(cube.corners.0, _mm_set1_epi8(0b01100000));
        let correct_ufr_corner_position = _mm_cmpeq_epi8(cube.corners.0, _mm_set1_epi8(0b01000000));

        let ufl_values = _mm_and_si128(correct_ufl_corner_position, _mm_setr_epi8( 0, 1, 0, 0, 0, 2, 0, 3, 0, 0, 0, 0, 0, 0, 0,0));
        let ufr_values = _mm_and_si128(correct_ufr_corner_position, _mm_setr_epi8( 4, 0, 0, 0, 8, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0,0));

        let edge_coord = _mm_sad_epu8(_mm_or_si128(ufl_values, ufr_values), _mm_set1_epi8(0));
        let corners = _mm_extract_epi16::<0>(edge_coord) as u8;

        let edges = (_mm_movemask_epi8(cube.edges.0) & 0xF) as u8;

        let coord = corners << 4 | edges;
        FRUDFinishCoord(coord)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
mod wasm32 {
    use std::arch::wasm32::{u8x16, u8x16_eq, v128_and, v128_or, u8x16_extract_lane, u8x16_bitmask};

    use crate::puzzles::c333::Cube333;
    use crate::puzzles::c333::steps::finish::coords::FRUDFinishCoord;
    use crate::wasm_util::{mm_sad_epu8, u8x16_set1};

    #[inline]
    pub fn from_fr_finish_coord(cube: &Cube333) -> FRUDFinishCoord {
        let correct_ufl_corner_position = u8x16_eq(cube.corners.0, u8x16_set1(0b01100000));
        let correct_ufr_corner_position = u8x16_eq(cube.corners.0, u8x16_set1(0b01000000));

        let ufl_values = v128_and(correct_ufl_corner_position, u8x16(0, 1, 0, 0, 0, 2, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0));
        let ufr_values = v128_and(correct_ufr_corner_position, u8x16(4, 0, 0, 0, 8, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0));

        let edge_coord = mm_sad_epu8(v128_or(ufl_values, ufr_values));
        let corners = u8x16_extract_lane::<0>(edge_coord);

        let edges = (u8x16_bitmask(cube.edges.0) & 0xF) as u8;

        let coord = corners << 4 | edges;
        FRUDFinishCoord(coord)
    }
}