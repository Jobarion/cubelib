#[cfg(feature = "cubic-odd")]
use crate::puzzles::cube::CubeCornersOdd;
use crate::steps::coord::Coord;

//UD corner orientation
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct COUDCoord(pub(crate) u16);

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

#[cfg(feature = "cubic-odd")]
impl From<&CubeCornersOdd> for COUDCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CubeCornersOdd) -> Self {
        unsafe { avx2::unsafe_from_cocoord(value.0) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CubeCornersOdd) -> Self {
        wasm32::from_cocoord(value)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_extract_epi16, _mm_hadd_epi16, _mm_mullo_epi16, _mm_set1_epi8, _mm_shuffle_epi8};

    use crate::simd_util::avx2::C;
    use crate::puzzles::cube::coords::COUDCoord;

    const CO_MUL: __m128i = unsafe { C { a_u16: [1, 3, 9, 27, 81, 243, 729, 0] }.a };
    const CO_SHUFFLE_8_TO_16: __m128i = unsafe { C { a_u8: [0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF] }.a };

    #[inline]
    pub(crate) unsafe fn unsafe_from_cocoord(value: __m128i) -> COUDCoord {
        //Spread co data out into 16bit values to avoid overflow later
        let co_epi16 = _mm_and_si128(
            _mm_shuffle_epi8(value, CO_SHUFFLE_8_TO_16),
            _mm_set1_epi8(0b11),
        );
        //Multiply with 3^0, 3^1, etc.
        let coord_values = _mm_mullo_epi16(co_epi16, CO_MUL);
        //Horizontal sum
        let coord = hsum_epi16_sse3(coord_values);
        COUDCoord(coord)
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
    use std::arch::wasm32::{u16x8, u16x8_extract_lane, u16x8_mul, u8x16, u8x16_swizzle, v128, v128_and};

    use crate::puzzles::cube::coords::COUDCoord;
    use crate::puzzles::cube::CubeCornersOdd;
    use crate::wasm_util::{complete_hsum_epi16, u8x16_set1};

    const CO_MUL: v128 = u16x8(1, 3, 9, 27, 81, 243, 729, 0);
    const CO_SHUFFLE_8_TO_16: v128 = u8x16(0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF);

    #[inline]
    pub(crate) fn from_cocoord(value: &CubeCornersOdd) -> COUDCoord {
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