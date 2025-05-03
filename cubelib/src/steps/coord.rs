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

#[cfg(target_feature = "avx2")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPCoord(pub(crate) u16);

#[cfg(target_feature = "avx2")]
impl Coord<40320> for CPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

#[cfg(target_feature = "avx2")]
impl Into<usize> for CPCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

#[cfg(target_feature = "avx2")]
impl From<&crate::cube::CornerCube333> for CPCoord {
    fn from(value: &crate::cube::CornerCube333) -> Self {
        unsafe {
            avx2::unsafe_from_cpcoord(value)
        }
    }
}

#[cfg(target_feature = "avx2")]
impl From<&crate::cube::Cube333> for CPCoord {
    fn from(value: &crate::cube::Cube333) -> Self {
        Self::from(&value.corners)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_cmplt_epi8, _mm_extract_epi16, _mm_hadd_epi16, _mm_hadd_epi32, _mm_mullo_epi16, _mm_set1_epi8, _mm_set_epi64x, _mm_setr_epi16, _mm_setr_epi8, _mm_shuffle_epi8, _mm_srli_epi32};
    use crate::cube::CornerCube333;
    use crate::steps::coord::CPCoord;

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
