use crate::coords::coord::Coord;
use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRFinishCoord(pub(crate) u8);

pub const FR_FINISH_SIZE: usize = 256;
impl Coord<{FR_FINISH_SIZE}> for FRFinishCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRFinishCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<&CubieCube> for FRFinishCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CubieCube) -> Self {
        unsafe { avx2::unsafe_from_fr_finish_coord(value) }
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_castpd_si128, _mm_castsi128_pd, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_cmplt_epi8, _mm_extract_epi16, _mm_hadd_epi32, _mm_movemask_epi8, _mm_mullo_epi16, _mm_or_si128, _mm_permute_pd, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi8, _mm_shuffle_epi8, _mm_srli_epi32, _mm_xor_si128};
    use crate::coords::finish::FRFinishCoord;
    use crate::coords::fr::{FRCPOrbitCoord, FREdgesCoord, FROrbitParityCoord, FRSliceEdgesCoord};
    use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_finish_coord(cube: &CubieCube) -> FRFinishCoord {

        // let cp_values = _mm_srli_epi32::<5>(cube.corners.0);
        //
        // let values246 = _mm_shuffle_epi8(
        //     cp_values,
        //     _mm_set_epi8(-1, -1, -1, -1, -1, 6, -1, -1, -1, 6, 4, -1, -1, 6, 4, 2),
        // );
        //
        // let higher_left_246 = _mm_and_si128(
        //     _mm_cmplt_epi8(
        //         values246,
        //         _mm_shuffle_epi8(
        //             cp_values,
        //             _mm_set_epi8(-1, -1, -1, -1, -1, 4, -1, -1, -1, 2, 2, -1, -1, 0, 0, 0),
        //         ),
        //     ),
        //     _mm_set1_epi8(1),
        // );
        //
        //
        // let result = _mm_hadd_epi32(higher_left_246, _mm_set1_epi8(0));
        // let result = _mm_hadd_epi32(result, _mm_set1_epi8(0));
        //
        // let result = _mm_shuffle_epi8(result, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 2, -1, 1, -1, 0));
        //
        // let factorials = _mm_set_epi16(0, 5040, 720, 120, 24, 6, 2, 1);
        //
        // let result = _mm_mullo_epi16(result, factorials);
        // let corners = _mm_extract_epi16::<0>(_mm_sad_epu8(result, _mm_set1_epi8(0))) as u16;

        let correct_ufl_corner_position = _mm_cmpeq_epi8(cube.corners.0, _mm_set1_epi8(0b01100000));
        let correct_ufr_corner_position = _mm_cmpeq_epi8(cube.corners.0, _mm_set1_epi8(0b01000000));

        let ufl_values = _mm_and_si128(correct_ufl_corner_position, _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 2, 0, 0, 0, 1, 0));
        let ufr_values = _mm_and_si128(correct_ufr_corner_position, _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 8, 0, 0, 0, 4));

        let edge_coord = _mm_sad_epu8(_mm_or_si128(ufl_values, ufr_values), _mm_set1_epi8(0));
        let corners = _mm_extract_epi16::<0>(edge_coord) as u8;

        let edges = (_mm_movemask_epi8(cube.edges.0) & 0xF) as u8;

        let coord = corners << 4 | edges;
        FRFinishCoord(coord)
    }
}