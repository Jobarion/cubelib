use crate::puzzles::c333::{Cube333, EdgeCube333};
use crate::puzzles::cube::coords::COUDCoord;
use crate::steps::coord::Coord;

//Coordinate representing the position of edges that belong into the UD slice.
//0 if they are in the slice, they don't have to be in the correct position
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct UDSliceUnsortedCoord(pub(crate) u16);

//Assuming we already have FB-EO, represents the combination of UDSliceUnsortedCoord and COUDCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DRUDEOFBCoord(pub(crate) u32);


impl Coord<495> for UDSliceUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for UDSliceUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

//TODO this should use 'impl const' once it's stable
pub const DRUDEOFB_SIZE: usize = 495 * 2187;
impl Coord<DRUDEOFB_SIZE> for DRUDEOFBCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for DRUDEOFBCoord {
    fn into(self) -> usize {
        self.val()
    }
}

impl From<&EdgeCube333> for UDSliceUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_udslice_unsorted_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCube333) -> Self {
        wasm32::from_udslice_unsorted_coord(value)
    }
}

impl From<&Cube333> for DRUDEOFBCoord {
    #[inline]
    fn from(value: &Cube333) -> Self {
        let ud_slice = UDSliceUnsortedCoord::from(&value.edges).val();
        let co = COUDCoord::from(&value.corners).val();
        let index = co * UDSliceUnsortedCoord::size() + ud_slice;
        DRUDEOFBCoord(index as u32)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_extract_epi16, _mm_hadd_epi32, _mm_or_si128, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_setr_epi32, _mm_setr_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_slli_si128, _mm_srli_epi32, _mm_sub_epi8};

    use crate::alignment::avx2::C;
    use crate::puzzles::c333::EdgeCube333;
    use crate::puzzles::c333::steps::dr::coords::UDSliceUnsortedCoord;

    const UD_SLICE_BINOM_0_ARR: [u8; 16] = [
        b(0, 0), b(0, 1), b(0, 2), b(0, 3),
        b(1, 0), b(1, 1), b(1, 2), b(1, 3),
        b(2, 0), b(2, 1), b(2, 2), b(2, 3),
        b(3, 0), b(3, 1), b(3, 2), b(3, 3),
    ];
    const UD_SLICE_BINOM_1_ARR: [u8; 16] = [
        b(4, 0), b(4, 1), b(4, 2), b(4, 3),
        b(5, 0), b(5, 1), b(5, 2), b(5, 3),
        b(6, 0), b(6, 1), b(6, 2), b(6, 3),
        b(7, 0), b(7, 1), b(7, 2), b(7, 3),
    ];
    const UD_SLICE_BINOM_2_ARR: [u8; 16] = [
        b(8, 0), b(8, 1), b(8, 2), b(8, 3),
        b(9, 0), b(9, 1), b(9, 2), b(9, 3),
        b(10, 0), b(10, 1), b(10, 2), b(10, 3),
        b(11, 0), b(11, 1), b(11, 2), b(11, 3),
    ];

    const UD_SLICE_BINOM_0: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_0_ARR, }.a };
    const UD_SLICE_BINOM_1: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_1_ARR, }.a };
    const UD_SLICE_BINOM_2: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_2_ARR, }.a };


    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_udslice_unsorted_coord(
        value: &EdgeCube333,
    ) -> UDSliceUnsortedCoord {
        let coord = unsafe {
            let slice_edges =
                _mm_srli_epi32::<6>(_mm_and_si128(value.0, _mm_set1_epi8(0b01000000)));
            //Our edge order is
            // UB UR UF UL FR FL BR BL DF DR DB DL

            //Kociemba uses
            // UR UF UL UB DR DF DL DB FR FL BL BR

            //We map to Kociemba's order here to make things simpler for us, but this could be optimized out if we just adjust the later shuffle masks
            let slice_edges = _mm_shuffle_epi8(
                slice_edges,
                _mm_setr_epi8( 1, 2, 3, 0, 9, 8, 11, 10, 4, 5, 7, 6, -1, -1, -1,-1),
            );

            let non_slice_edge_mask = _mm_cmpeq_epi8(slice_edges, _mm_set1_epi8(0));

            let e0123 = _mm_shuffle_epi8(
                slice_edges,
                _mm_setr_epi8( 0, 0, 0, 0, -1, 1, 1, 1, -1, -1, 2, 2, -1, -1, -1,3),
            );
            let e4567 = _mm_shuffle_epi8(
                slice_edges,
                _mm_setr_epi8( 4, 4, 4, 4, -1, 5, 5, 5, -1, -1, 6, 6, -1, -1, -1,7),
            );
            let e891011 = _mm_shuffle_epi8(
                slice_edges,
                _mm_setr_epi8( 8, 8, 8, 8, -1, 9, 9, 9, -1, -1, 10, 10, -1, -1, -1,11),
            );

            let hadd = _mm_hadd_epi32(e0123, e4567);
            let hadd = _mm_hadd_epi32(hadd, e891011);
            let hadd0123 = _mm_and_si128(hadd, _mm_setr_epi32( -1, 0, 0,0));

            let hadd4567891011 = _mm_hadd_epi32(
                _mm_shuffle_epi8(
                    hadd,
                    _mm_setr_epi8( 3, 3, 3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,15),
                ),
                _mm_set1_epi8(0),
            );
            let hadd4567891011 = _mm_add_epi8(
                hadd4567891011,
                _mm_shuffle_epi8(
                    hadd4567891011,
                    _mm_setr_epi8( 15, 15, 15, 15, 3, 3, 3, 3, -1, -1, -1, -1, -1, -1, -1,-1),
                ),
            );

            let hadd = _mm_or_si128(_mm_slli_si128::<4>(hadd4567891011), hadd0123);
            let hadd = _mm_and_si128(hadd, non_slice_edge_mask);

            let lut_index = _mm_and_si128(
                _mm_sub_epi8(hadd, _mm_set1_epi8(1)),
                _mm_set1_epi8(0b10001111_u8 as i8),
            );
            let lut_index = _mm_add_epi8(
                lut_index,
                _mm_setr_epi8( 0, 4, 8, 12, 0, 4, 8, 12, 0, 4, 8, 12, 0, 0, 0,0),
            );

            let binom0123 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index),
                _mm_setr_epi32( -1, 0, 0,0),
            );
            let binom4567 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index),
                _mm_setr_epi32( 0, -1, 0,0),
            );
            let binom891011 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_2, lut_index),
                _mm_setr_epi32( 0, 0, -1,0),
            );

            let hsum = _mm_or_si128(binom0123, _mm_or_si128(binom4567, binom891011));

            let hsum_u16 = _mm_sad_epu8(hsum, _mm_set1_epi8(0));

            let hsum = _mm_hadd_epi32(_mm_shuffle_epi32::<0b11111000>(hsum_u16), _mm_set1_epi32(0));

            _mm_extract_epi16::<0>(hsum) as u16
        };
        UDSliceUnsortedCoord(coord)
    }

    const FACTORIAL: [u32; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
    ];

    const fn b(n: u8, k: u8) -> u8 {
        if n == 0 || n < k {
            return 0;
        }
        (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
    }
}

#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
mod wasm32 {
    use std::arch::wasm32::{i32x4, i8x16, u16x8, u16x8_extract_lane, u16x8_mul, u32x4_shr, u32x4_shuffle, u8x16, u8x16_add, u8x16_eq, u8x16_sub, u8x16_swizzle, v128, v128_and, v128_or};

    use crate::puzzles::c333::{CornerCube333, EdgeCube333};
    use crate::puzzles::c333::steps::dr::coords::UDSliceUnsortedCoord;
    use crate::wasm_util::{complete_hsum_epi16, hsum_narrow_epi32, hsum_wide_epi32, mm_sad_epu8, u8x16_set1};

    const UD_SLICE_BINOM_0: v128 = u8x16(
        b(0, 0), b(0, 1), b(0, 2), b(0, 3),
        b(1, 0), b(1, 1), b(1, 2), b(1, 3),
        b(2, 0), b(2, 1), b(2, 2), b(2, 3),
        b(3, 0), b(3, 1), b(3, 2), b(3, 3),
    );
    const UD_SLICE_BINOM_1: v128 = u8x16(
        b(4, 0), b(4, 1), b(4, 2), b(4, 3),
        b(5, 0), b(5, 1), b(5, 2), b(5, 3),
        b(6, 0), b(6, 1), b(6, 2), b(6, 3),
        b(7, 0), b(7, 1), b(7, 2), b(7, 3),
    );
    const UD_SLICE_BINOM_2: v128 = u8x16(
        b(8, 0), b(8, 1), b(8, 2), b(8, 3),
        b(9, 0), b(9, 1), b(9, 2), b(9, 3),
        b(10, 0), b(10, 1), b(10, 2), b(10, 3),
        b(11, 0), b(11, 1), b(11, 2), b(11, 3),
    );

    #[inline]
    pub(crate) fn from_udslice_unsorted_coord(
        value: &EdgeCube333,
    ) -> UDSliceUnsortedCoord {
        let coord = {
            let slice_edges =
                u32x4_shr(v128_and(value.0, u8x16_set1(0b01000000)), 6);
            //Our edge order is
            // UB UR UF UL FR FL BR BL DF DR DB DL

            //Kociemba uses
            // UR UF UL UB DR DF DL DB FR FL BL BR

            //We map to Kociemba's order here to make things simpler for us, but this could be optimized out if we just adjust the later shuffle masks
            let slice_edges = u8x16_swizzle(
                slice_edges,
                i8x16(1, 2, 3, 0, 9, 8, 11, 10, 4, 5, 7, 6, -1, -1, -1, -1)
            );

            let non_slice_edge_mask = u8x16_eq(slice_edges, u8x16_set1(0));

            let e0123 = u8x16_swizzle(
                slice_edges,
                i8x16(0, 0, 0, 0, -1, 1, 1, 1, -1, -1, 2, 2, -1, -1, -1, 3)
            );
            let e4567 = u8x16_swizzle(
                slice_edges,
                i8x16(4, 4, 4, 4, -1, 5, 5, 5, -1, -1, 6, 6, -1, -1, -1, 7)
            );
            let e891011 = u8x16_swizzle(
                slice_edges,
                i8x16(8, 8, 8, 8, -1, 9, 9, 9, -1, -1, 10, 10, -1, -1, -1, 11)
            );

            let hadd = hsum_wide_epi32(e0123, e4567);
            let hadd = hsum_wide_epi32(hadd, e891011);
            let hadd0123 = v128_and(hadd, i32x4(-1, 0, 0, 0));

            let hadd4567891011 = hsum_narrow_epi32(
                u8x16_swizzle(
                    hadd,
                    u8x16(3, 3, 3, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15)
                ),
            );
            let hadd4567891011 = u8x16_add(
                hadd4567891011,
                u8x16_swizzle(
                    hadd4567891011,
                    i8x16(15, 15, 15, 15, 3, 3, 3, 3, -1, -1, -1, -1, -1, -1, -1, -1)
                ),
            );

            let hadd = v128_or(u32x4_shuffle::<4, 0, 1, 2>(hadd4567891011, i32x4(0, 0, 0, 0)), hadd0123);

            let hadd = v128_and(hadd, non_slice_edge_mask);

            let lut_index = v128_and(
                u8x16_sub(hadd, u8x16_set1(1)),
                u8x16_set1(0b10001111),
            );
            let lut_index = u8x16_add(
                lut_index,
                u8x16(0, 4, 8, 12, 0, 4, 8, 12, 0, 4, 8, 12, 0, 0, 0, 0)
            );

            let binom0123 = v128_and(
                u8x16_swizzle(UD_SLICE_BINOM_0, lut_index),
                i32x4(-1, 0, 0, 0),
            );
            let binom4567 = v128_and(
                u8x16_swizzle(UD_SLICE_BINOM_1, lut_index),
                i32x4(0, -1, 0, 0),
            );
            let binom891011 = v128_and(
                u8x16_swizzle(UD_SLICE_BINOM_2, lut_index),
                i32x4(0, 0, -1, 0),
            );

            let hsum = v128_or(binom0123, v128_or(binom4567, binom891011));

            u16x8_extract_lane::<0>(mm_sad_epu8(hsum))
        };
        UDSliceUnsortedCoord(coord)
    }

    const FACTORIAL: [u32; 12] = [
        1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800,
    ];

    const fn b(n: u8, k: u8) -> u8 {
        if n == 0 || n < k {
            return 0;
        }
        (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
    }
}