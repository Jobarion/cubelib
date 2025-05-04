use std::hash::Hash;

pub trait Coord<const SIZE: usize>: Into<usize> + Copy + Clone + Eq + PartialEq + Hash {
    fn size() -> usize {
        SIZE
    }
    fn val(&self) -> usize;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ZeroCoord;

impl Into<usize> for ZeroCoord {
    fn into(self) -> usize {
        0
    }
}

impl Coord<0> for ZeroCoord {
    fn val(&self) -> usize {
        0
    }
}

impl <P> From<&P> for ZeroCoord {
    fn from(_: &P) -> Self {
        ZeroCoord
    }
}

#[cfg(any(target_feature = "avx2", target_feature = "neon"))]
pub type CPCoord = default_coords::CPCoord;

#[cfg(any(target_feature = "avx2", target_feature = "neon"))]
mod default_coords {
    use crate::cube::{CornerCube333, Cube333};
    use crate::steps;
    use crate::steps::coord::{Coord};

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
    pub struct CPCoord(pub(crate) u16);

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

    impl From<&CornerCube333> for CPCoord {
        fn from(value: &CornerCube333) -> Self {
            #[cfg(target_feature = "avx2")]
            unsafe {
                steps::coord::avx2::unsafe_from_cpcoord(value)
            }
            #[cfg(target_feature = "neon")]
            unsafe {
                steps::coord::neon::unsafe_from_cpcoord(value)
            }
        }
    }

    impl From<&Cube333> for CPCoord {
        fn from(value: &Cube333) -> Self {
            Self::from(&value.corners)
        }
    }
}

#[cfg(target_feature = "avx2")]
pub(crate) mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_cmplt_epi8, _mm_extract_epi16, _mm_hadd_epi16, _mm_hadd_epi32, _mm_mullo_epi16, _mm_set1_epi8, _mm_set_epi64x, _mm_setr_epi16, _mm_setr_epi8, _mm_shuffle_epi8, _mm_srli_epi32};

    use crate::cube::CornerCube333;
    use crate::steps::coord::default_coords::CPCoord;

    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn unsafe_from_cpcoord(value: &CornerCube333) -> CPCoord {
        let cp_values = _mm_and_si128(_mm_srli_epi32::<5>(value.0), _mm_set1_epi8(0b111));
        CPCoord(unsafe_permutation_8(cp_values))
    }

    #[target_feature(enable = "avx2")]
    pub(crate) unsafe fn unsafe_permutation_8(val: __m128i) -> u16 {

        //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
        let values_67 = _mm_shuffle_epi8(
            val,
            _mm_setr_epi8( 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, -1, 7, -1,-1),
        );
        let values_2345 = _mm_shuffle_epi8(
            val,
            _mm_setr_epi8( 2, 3, 4, 5, 2, 3, 4, 5, -1, 3, 4, 5, -1, -1, 4,5),
        );
        let values_15 = _mm_shuffle_epi8(val, _mm_set_epi64x(5, 1));

        let higher_left_67 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_67,
                _mm_shuffle_epi8(
                    val,
                    _mm_setr_epi8( 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, -1, 6, -1,-1),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_2345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_2345,
                _mm_shuffle_epi8(
                    val,
                    _mm_setr_epi8( 0, 0, 0, 0, 1, 1, 1, 1, -1, 2, 2, 2, -1, -1, 3,3),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_15 = _mm_and_si128(
            _mm_cmplt_epi8(values_15, _mm_shuffle_epi8(val, _mm_set_epi64x(4, 0))),
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

        hsum_epi16_sse3(prod)
    }

    #[inline]
    unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
        let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        _mm_extract_epi16::<0>(sum) as u16
    }
}

#[cfg(target_feature = "neon")]
pub(crate) mod neon {
    use std::arch::aarch64::{uint8x8_t, vaddq_u8, vaddvq_u16, vandq_u8, vcltq_u8, vcombine_u8, vdup_n_u8, vdupq_n_u8, vmulq_u16, vorrq_u8, vqtbl1q_u8, vreinterpretq_u16_u8};

    use crate::cube::CornerCube333;
    use crate::simd_util::neon::C16;
    use crate::steps::coord::default_coords::CPCoord;

    pub(crate) unsafe fn unsafe_from_cpcoord(cube: &CornerCube333) -> CPCoord {
        CPCoord(unsafe_permutation_8(cube.0))
    }

    pub(crate) unsafe fn unsafe_permutation_8(val: uint8x8_t) -> u16 {
        let val = vcombine_u8(val, vdup_n_u8(0));
        let values_367 = vqtbl1q_u8(
            val,
            C16{ a_i8: [3, 7, 6, 3, 7, 6, 3, 7, 6, 7, 6, 7, 6, 7, 6, 7] }.a
        );
        let higher_left_367 = vandq_u8(vcltq_u8(
            values_367,
            C16 { a_i8: [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 5, 6] }.a
        ), vdupq_n_u8(1));

        let sum_367 = vaddq_u8(
            higher_left_367,
            vqtbl1q_u8(higher_left_367, C16 { a_i8: [3, -1, 5, -1, 1, -1, -1, 9, 10, -1, -1, 13, 14, -1, -1, -1]}.a)
        );
        let sum_367 = vaddq_u8(
            sum_367,
            vqtbl1q_u8(sum_367, C16 { a_i8: [6, -1, 8, -1, 7, -1, -1, -1, -1, -1, -1, 15, -1, -1, -1, -1]}.a)
        );
        let sum_367 = vaddq_u8(
            sum_367,
            vqtbl1q_u8(sum_367, C16 { a_i8: [-1, -1, 12, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1]}.a)
        );

        let values_1245 = vqtbl1q_u8(
            val,
            C16 { a_i8: [1, 2, 2, 4, 4, 4, 5, 4, 5, 5, 5, 5, -1, -1, -1, -1] }.a
        );
        let higher_left_1245 = vandq_u8(vcltq_u8(
            values_1245,
            C16 { a_i8: [0, 0, 1, 0, 1, 2, 0, 3, 1, 2, 3, 4, -1, -1, -1, -1] }.a
        ), vdupq_n_u8(1));

        let sum_1245 = vaddq_u8(
            higher_left_1245,
            vqtbl1q_u8(higher_left_1245, C16 { a_i8: [-1, -1, 1, -1, 3, 7, 8, -1, -1, 10, -1, -1, -1, -1, -1, -1]}.a)
        );
        let sum_1245 = vaddq_u8(
            sum_1245,
            vqtbl1q_u8(sum_1245, C16 { a_i8: [-1, -1, -1, -1, 5, -1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1]}.a)
        );
        let sum_1245 = vaddq_u8(
            sum_1245,
            vqtbl1q_u8(sum_1245, C16 { a_i8: [-1, -1, -1, -1, -1, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1]}.a)
        );
        let sum_1245 = vqtbl1q_u8(
            sum_1245,
            C16 { a_i8: [0, -1, 2, -1, -1, -1, 4, -1, 6, -1, -1, -1, -1, -1, -1, -1]}.a
        );
        let sum_367 = vqtbl1q_u8(
            sum_367,
            C16 { a_i8: [-1, -1, -1, -1, 0, -1, -1, -1, -1, -1, 2, -1, 4, -1, -1, -1]}.a
        );
        let sum = vreinterpretq_u16_u8(vorrq_u8(sum_1245, sum_367));
        let mul = vmulq_u16(sum, C16{ a_u16: [1, 2, 6, 24, 120, 720, 5040, 0]}.a_16);
        vaddvq_u16(mul)
    }
}
