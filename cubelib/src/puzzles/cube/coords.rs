use crate::puzzles::cube::CornerCube;
use crate::steps::coord::Coord;

//UD corner orientation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct COUDCoord(pub(crate) u16);
//Corner permutation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPCoord(pub(crate) u16);

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

impl Coord<40320> for CPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<&CornerCube> for COUDCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCube) -> Self {
        unsafe { avx2::unsafe_from_cocoord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CornerCube333) -> Self {
        wasm32::from_cocoord(value)
    }
}

impl From<&CornerCube> for CPCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCube) -> Self {
        unsafe { avx2::unsafe_from_cpcoord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CornerCube) -> Self {
        wasm32::from_cpcoord(value)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_cmplt_epi8, _mm_extract_epi16, _mm_hadd_epi16, _mm_hadd_epi32, _mm_mullo_epi16, _mm_set1_epi8, _mm_set_epi64x, _mm_setr_epi16, _mm_setr_epi8, _mm_shuffle_epi8, _mm_srli_epi32};

    use crate::alignment::avx2::C;
    use crate::puzzles::cube::coords::{COUDCoord, CPCoord};
    use crate::puzzles::cube::CornerCube;

    const CO_MUL: __m128i = unsafe { C { a_u16: [1, 3, 9, 27, 81, 243, 729, 0] }.a };
    const CO_SHUFFLE_8_TO_16: __m128i = unsafe { C { a_u8: [0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF] }.a };

    #[inline]
    pub(crate) unsafe fn unsafe_from_cocoord(value: &CornerCube) -> COUDCoord {
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
    pub(crate) unsafe fn unsafe_from_cpcoord(value: &CornerCube) -> CPCoord {
        let cp_values = _mm_and_si128(_mm_srli_epi32::<5>(value.0), _mm_set1_epi8(0b111));

        //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
        let values_67 = _mm_shuffle_epi8(
            cp_values,
            _mm_setr_epi8( 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, -1, 7, -1,-1),
        );
        let values_2345 = _mm_shuffle_epi8(
            cp_values,
            _mm_setr_epi8( 2, 3, 4, 5, 2, 3, 4, 5, -1, 3, 4, 5, -1, -1, 4,5),
        );
        let values_15 = _mm_shuffle_epi8(cp_values, _mm_set_epi64x(5, 1));

        let higher_left_67 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_67,
                _mm_shuffle_epi8(
                    cp_values,
                    _mm_setr_epi8( 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, -1, 6, -1,-1),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_2345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_2345,
                _mm_shuffle_epi8(
                    cp_values,
                    _mm_setr_epi8( 0, 0, 0, 0, 1, 1, 1, 1, -1, 2, 2, 2, -1, -1, 3,3),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_15 = _mm_and_si128(
            _mm_cmplt_epi8(values_15, _mm_shuffle_epi8(cp_values, _mm_set_epi64x(4, 0))),
            _mm_set1_epi8(1),
        );

        let hsum = _mm_hadd_epi32(higher_left_2345, higher_left_67);
        let hsum = _mm_hadd_epi32(hsum, higher_left_15);
        let hsum = _mm_shuffle_epi8(
            hsum,
            _mm_setr_epi8( 8, 0, -1, -1, 1, 2, -1, -1, 3, 4, 12, 6, 5, -1, 7,-1),
        );
        let hsum = _mm_hadd_epi16(hsum, _mm_set1_epi8(0));
        let hsum = _mm_shuffle_epi8(
            hsum,
            _mm_setr_epi8( 0, -1, 1, -1, 2, -1, 3, -1, 4, -1, 5, -1, 6, -1, -1,-1),
        );
        let factorials = _mm_setr_epi16( 1, 2, 6, 24, 120, 720, 5040,0);
        let prod = _mm_mullo_epi16(hsum, factorials);

        CPCoord(hsum_epi16_sse3(prod))
    }

    #[inline]
    unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
        let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        _mm_extract_epi16::<0>(sum) as u16
    }
}

#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
mod wasm32 {
    use std::arch::wasm32::{i32x4, i8x16, u16x8, u16x8_extract_lane, u16x8_mul, u32x4_shr, u32x4_shuffle, u8x16, u8x16_add, u8x16_eq, u8x16_sub, u8x16_swizzle, v128, v128_and, v128_or};

    use crate::puzzles::cube::coords::{COUDCoord, CPCoord};
    use crate::puzzles::cube::CornerCube;

    use crate::wasm_util::{complete_hsum_epi16, hsum_narrow_epi32, hsum_wide_epi32, mm_sad_epu8, u8x16_set1};

    const CO_MUL: v128 = u16x8(1, 3, 9, 27, 81, 243, 729, 0);
    const CO_SHUFFLE_8_TO_16: v128 = u8x16(0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF);

    #[inline]
    pub(crate) fn from_cocoord(value: &CornerCube) -> COUDCoord {
        //Spread co data out into 16bit values to avoid overflow later
        let co_epi16 = v128_and(
            u8x16_swizzle(value.0, CO_SHUFFLE_8_TO_16),
            u8x16_set1(0b11),
        );
        //Multiply with 3^0, 3^1, etc.
        let coord_values = u16x8_mul(co_epi16, CO_MUL);
        //Horizontal sum
        let coord = u16x8_extract_lane::<0>(complete_hsum_epi16(coord_values));
        COUDCoord(coord)
    }
}