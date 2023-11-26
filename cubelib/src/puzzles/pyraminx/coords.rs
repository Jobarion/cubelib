use crate::puzzles::pyraminx::Pyraminx;
use crate::steps::coord::Coord;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EOCoordLR(u8);

impl Coord<32> for EOCoordLR {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EOCoordLR {
    fn into(self) -> usize {
        self.val()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CenterCoord(u8);

impl Coord<81> for CenterCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CenterCoord {
    fn into(self) -> usize {
        self.val()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EPCoord(u16);

impl Coord<360> for EPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EPCoord {
    fn into(self) -> usize {
        self.val()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NoTipsCoord(u32);

pub const NO_TIPS_COORD_SIZE: usize = 360*81*32;
impl Coord<{ NO_TIPS_COORD_SIZE }> for NoTipsCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for NoTipsCoord {
    fn into(self) -> usize {
        self.val()
    }
}

impl From<&Pyraminx> for EOCoordLR {
    fn from(value: &Pyraminx) -> Self {
        #[cfg(target_feature = "avx2")]
        unsafe { avx2::eocoord_lr(value) }
    }
}

impl From<&Pyraminx> for CenterCoord {
    fn from(value: &Pyraminx) -> Self {
        #[cfg(target_feature = "avx2")]
        unsafe { avx2::centercoord(value) }
    }
}

impl From<&Pyraminx> for EPCoord {
    fn from(value: &Pyraminx) -> Self {
        #[cfg(target_feature = "avx2")]
        unsafe { avx2::epcoord(value) }
    }
}

impl From<&Pyraminx> for NoTipsCoord {
    fn from(value: &Pyraminx) -> Self {
        let eo = EOCoordLR::from(value).val();
        let ep = EPCoord::from(value).val();
        let center = CenterCoord::from(value).val();
        let coord = EOCoordLR::size() * EPCoord::size() * center + EOCoordLR::size() * ep + eo;
        NoTipsCoord(coord as u32)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_cmplt_epi8, _mm_extract_epi16, _mm_extract_epi8, _mm_hadd_epi16, _mm_hadd_epi32, _mm_movemask_epi8, _mm_mullo_epi16, _mm_sad_epu8, _mm_set1_epi8, _mm_setr_epi16, _mm_setr_epi8, _mm_shuffle_epi8, _mm_slli_epi64, _mm_srli_epi32};

    use crate::puzzles::pyraminx::coords::{CenterCoord, EOCoordLR, EPCoord};
    use crate::puzzles::pyraminx::Pyraminx;

    pub unsafe fn eocoord_lr(pyra: &Pyraminx) -> EOCoordLR {
        let no_db_edge = _mm_and_si128(
            pyra.0,
            _mm_setr_epi8(0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
        );
        let lr = _mm_movemask_epi8(_mm_slli_epi64::<7>(no_db_edge)) as u8;
        EOCoordLR(lr)
    }

    pub unsafe fn centercoord(pyra: &Pyraminx) -> CenterCoord {
        let values = _mm_mullo_epi16(
            _mm_shuffle_epi8(pyra.0, _mm_setr_epi8(8, -1, 9, -1, 10, -1, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1)),
            _mm_setr_epi16(1, 3, 9, 27, 0, 0, 0, 0)
        );
        let sum = _mm_sad_epu8(values, _mm_set1_epi8(0));
        CenterCoord(_mm_extract_epi8::<0>(sum) as u8)
    }

    pub(crate) unsafe fn epcoord(value: &Pyraminx) -> EPCoord {
        let ep_values = _mm_and_si128(_mm_srli_epi32::<1>(value.0), _mm_set1_epi8(0b111));


        let values_2345 = _mm_shuffle_epi8(
            ep_values,
            _mm_setr_epi8(2, 3, 4, 5, 2, 3, 4, 5, -1, 3, 4, 5, -1, -1, 4, 5)
        );

        let values_5 = _mm_shuffle_epi8(
            ep_values,
            _mm_setr_epi8(-1, -1, -1, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
        );
        let higher_left_2345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_2345,
                _mm_shuffle_epi8(
                    ep_values,
                    _mm_setr_epi8( 0, 0, 0, 0, 1, 1, 1, 1, -1, 2, 2, 2, -1, -1, 3,3),
                ),
            ),
            _mm_set1_epi8(1),
        );
        let higher_left_5 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_5,
                _mm_shuffle_epi8(
                    ep_values,
                    _mm_setr_epi8(-1, -1, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
                ),
            ),
            _mm_set1_epi8(1),
        );

        let higher_left_2345 = _mm_hadd_epi32(higher_left_2345, _mm_set1_epi8(0));
        let higher_left_2345 = _mm_hadd_epi32(higher_left_2345, _mm_set1_epi8(0));
        let values = _mm_add_epi8(higher_left_2345, higher_left_5);
        let values_epu16 = _mm_shuffle_epi8(values, _mm_setr_epi8(0, -1, 1, -1, 2, -1, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1));
        // println!("{values:?}");
        let factorials = _mm_setr_epi16(2, 6, 24, 120, 0, 0, 0, 0);
        let prod = _mm_mullo_epi16(values_epu16, factorials);
        EPCoord(hsum_epi16_sse3(prod) >> 1)
    }

    #[inline]
    unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
        let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
        _mm_extract_epi16::<0>(sum) as u16
    }

}