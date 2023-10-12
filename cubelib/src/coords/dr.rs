use crate::coords::coord::Coord;
use crate::cube::{Corner, Invertible};
use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

//UD corner orientation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct COUDCoord(pub(crate) u16);

//Coordinate representing the position of edges that belong into the UD slice.
//0 if they are in the slice, they don't have to be in the correct position
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct UDSliceUnsortedCoord(pub(crate) u16);

//Assuming we already have FB-EO, represents the combination of UDSliceUnsortedCoord and COUDCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DRUDEOFBCoord(pub(crate) u32);

impl Coord<2187> for COUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for COUDCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

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

impl From<&CornerCubieCube> for COUDCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_cocoord(value) }
    }
}

impl From<&[Corner; 8]> for COUDCoord {
    fn from(value: &[Corner; 8]) -> Self {
        let mut co = 0_u16;

        for i in (0..7).rev() {
            co = co * 3 + value[i].orientation as u16;
        }

        COUDCoord(co)
    }
}

impl From<&EdgeCubieCube> for UDSliceUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_udslice_unsorted_coord(value) }
    }
}

impl From<&CubieCube> for DRUDEOFBCoord {
    #[inline]
    fn from(value: &CubieCube) -> Self {
        let ud_slice = UDSliceUnsortedCoord::from(&value.edges).val();
        let co = COUDCoord::from(&value.corners).val();
        let index = co * UDSliceUnsortedCoord::size() + ud_slice;
        DRUDEOFBCoord(index as u32)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_extract_epi16, _mm_hadd_epi16, _mm_hadd_epi32, _mm_mullo_epi16, _mm_or_si128, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi32, _mm_set_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_slli_si128, _mm_srli_epi32, _mm_sub_epi8};

    use crate::alignment::avx2::C;
    use crate::coords::dr::{COUDCoord, UDSliceUnsortedCoord};
    use crate::cubie::{CornerCubieCube, EdgeCubieCube};

    const UD_SLICE_BINOM_0_ARR: [u8; 16] = [
        b(0, 0),
        b(0, 1),
        b(0, 2),
        b(0, 3),
        b(1, 0),
        b(1, 1),
        b(1, 2),
        b(1, 3),
        b(2, 0),
        b(2, 1),
        b(2, 2),
        b(2, 3),
        b(3, 0),
        b(3, 1),
        b(3, 2),
        b(3, 3),
    ];
    const UD_SLICE_BINOM_1_ARR: [u8; 16] = [
        b(4, 0),
        b(4, 1),
        b(4, 2),
        b(4, 3),
        b(5, 0),
        b(5, 1),
        b(5, 2),
        b(5, 3),
        b(6, 0),
        b(6, 1),
        b(6, 2),
        b(6, 3),
        b(7, 0),
        b(7, 1),
        b(7, 2),
        b(7, 3),
    ];
    const UD_SLICE_BINOM_2_ARR: [u8; 16] = [
        b(8, 0),
        b(8, 1),
        b(8, 2),
        b(8, 3),
        b(9, 0),
        b(9, 1),
        b(9, 2),
        b(9, 3),
        b(10, 0),
        b(10, 1),
        b(10, 2),
        b(10, 3),
        b(11, 0),
        b(11, 1),
        b(11, 2),
        b(11, 3),
    ];

    const UD_SLICE_BINOM_0: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_0_ARR, }.a };
    const UD_SLICE_BINOM_1: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_1_ARR, }.a };
    const UD_SLICE_BINOM_2: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_2_ARR, }.a };

    const CO_MUL: __m128i = unsafe { C { a_u16: [1, 3, 9, 27, 81, 243, 729, 0] }.a };
    const CO_SHUFFLE_8_TO_16: __m128i = unsafe { C { a_u8: [0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF] }.a };


    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_udslice_unsorted_coord(
        value: &EdgeCubieCube,
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
                _mm_set_epi8(-1, -1, -1, -1, 6, 7, 5, 4, 10, 11, 8, 9, 0, 3, 2, 1),
            );

            let non_slice_edge_mask = _mm_cmpeq_epi8(slice_edges, _mm_set1_epi8(0));

            let e0123 = _mm_shuffle_epi8(
                slice_edges,
                _mm_set_epi8(3, -1, -1, -1, 2, 2, -1, -1, 1, 1, 1, -1, 0, 0, 0, 0),
            );
            let e4567 = _mm_shuffle_epi8(
                slice_edges,
                _mm_set_epi8(7, -1, -1, -1, 6, 6, -1, -1, 5, 5, 5, -1, 4, 4, 4, 4),
            );
            let e891011 = _mm_shuffle_epi8(
                slice_edges,
                _mm_set_epi8(11, -1, -1, -1, 10, 10, -1, -1, 9, 9, 9, -1, 8, 8, 8, 8),
            );

            let hadd = _mm_hadd_epi32(e0123, e4567);
            let hadd = _mm_hadd_epi32(hadd, e891011);
            let hadd0123 = _mm_and_si128(hadd, _mm_set_epi32(0, 0, 0, -1));

            let hadd4567891011 = _mm_hadd_epi32(
                _mm_shuffle_epi8(
                    hadd,
                    _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 3, 3, 3),
                ),
                _mm_set1_epi8(0),
            );
            let hadd4567891011 = _mm_add_epi8(
                hadd4567891011,
                _mm_shuffle_epi8(
                    hadd4567891011,
                    _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, 15, 15, 15, 15),
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
                _mm_set_epi8(0, 0, 0, 0, 12, 8, 4, 0, 12, 8, 4, 0, 12, 8, 4, 0),
            );

            let binom0123 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index),
                _mm_set_epi32(0, 0, 0, -1),
            );
            let binom4567 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index),
                _mm_set_epi32(0, 0, -1, 0),
            );
            let binom891011 = _mm_and_si128(
                _mm_shuffle_epi8(UD_SLICE_BINOM_2, lut_index),
                _mm_set_epi32(0, -1, 0, 0),
            );

            let hsum = _mm_or_si128(binom0123, _mm_or_si128(binom4567, binom891011));

            let hsum_u16 = _mm_sad_epu8(hsum, _mm_set1_epi8(0));

            let hsum = _mm_hadd_epi32(_mm_shuffle_epi32::<0b11111000>(hsum_u16), _mm_set1_epi32(0));

            _mm_extract_epi16::<0>(hsum) as u16
        };
        UDSliceUnsortedCoord(coord)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cocoord(value: &CornerCubieCube) -> COUDCoord {
        //Spread co data out into 16bit values to avoid overflow later
        let co_epi16 = _mm_and_si128(
            _mm_shuffle_epi8(value.0, CO_SHUFFLE_8_TO_16),
            _mm_set1_epi8(0b11),
        );
        //Multiply with 3^0, 3^1, etc.
        let coord_values = _mm_mullo_epi16(co_epi16, CO_MUL);
        //Horizontal sum
        let coord = hsum_epi16_sse3(coord_values);
        COUDCoord(coord)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
        let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        _mm_extract_epi16::<0>(sum) as u16
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