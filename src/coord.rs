use std::arch::x86_64::{__m128, __m128i, _mm256_add_epi32, _mm256_and_si256, _mm256_broadcastsi128_si256, _mm256_castsi128_si256, _mm256_castsi256_si128, _mm256_cmpgt_epi8, _mm256_extractf128_ps, _mm256_extracti128_si256, _mm256_hadd_epi16, _mm256_hadd_epi32, _mm256_mul_epi32, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi64x, _mm256_set1_epi8, _mm256_set_epi32, _mm256_set_epi64x, _mm256_set_epi8, _mm256_setr_epi8, _mm256_setr_m128, _mm256_setr_m128i, _mm256_shuffle_epi32, _mm256_shuffle_epi8, _mm_add_epi32, _mm_add_epi64, _mm_add_epi8, _mm_add_ss, _mm_and_si128, _mm_castpd_si128, _mm_castps_si128, _mm_castsi128_ps, _mm_cmpgt_epi8, _mm_cmplt_epi8, _mm_cvtss_si32, _mm_extract_epi16, _mm_extract_epi32, _mm_hadd_epi16, _mm_hadd_epi32, _mm_insert_epi8, _mm_movehdup_ps, _mm_movehl_ps, _mm_movemask_epi8, _mm_mullo_epi16, _mm_mullo_epi32, _mm_or_si128, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_shufflelo_epi16, _mm_slli_epi64, _mm_srli_epi32};
use crate::alignment::C;
use crate::cube::{Corner, Edge};
use crate::cubie::{CornerCubieCube, EdgeCubieCube};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EOCoord(pub u16, pub u16, pub u16);

impl From<EdgeCubieCube> for EOCoord {
    fn from(value: EdgeCubieCube) -> Self {
        unsafe {
            //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
            let no_db_edge = _mm_and_si128(value.0, _mm_set_epi8(0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F));
            let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
            let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
            let rl = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
            EOCoord(ud, fb, rl)
        }
    }
}

impl From<[Edge; 12]> for EOCoord {
    fn from(value: [Edge; 12]) -> Self {
        let mut ud = 0_u16;
        let mut fb = 0_u16;
        let mut rl = 0_u16;

        for i in (0..11).rev() {
            let edge = value[i];
            ud = (ud << 1) | (!edge.oriented_ud as u16);
            fb = (fb << 1) | (!edge.oriented_fb as u16);
            rl = (rl << 1) | (!edge.oriented_rl as u16);
        }

        EOCoord(ud, fb, rl)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct COCoord(pub u16);

impl COCoord {
    const CO_MUL: __m128i = unsafe { C { a_u16: [1, 3, 9, 27, 81, 243, 729, 0] }.a };
    const CO_SHUFFLE_8_TO_16: __m128i = unsafe { C { a_u8: [0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF] }.a };
}

impl From<CornerCubieCube> for COCoord {
    fn from(value: CornerCubieCube) -> Self {
        unsafe {
            //Spread co data out into 16bit values to avoid overflow later
            let co_epi16 = _mm_and_si128(_mm_shuffle_epi8(value.0, COCoord::CO_SHUFFLE_8_TO_16), _mm_set1_epi8(0b11));
            //Multiply with 3^0, 3^1, etc.
            let coord_values = _mm_mullo_epi16(co_epi16, COCoord::CO_MUL);
            //Horizontal sum
            let coord = hsum_epi16_sse3(coord_values);

            COCoord(coord)
        }
    }
}

impl From<[Corner; 8]> for COCoord {
    fn from(value: [Corner; 8]) -> Self {
        let mut co = 0_u16;

        for i in (0..7).rev() {
            co = co * 3 + value[i].orientation as u16;
        }

        COCoord(co)
    }
}

unsafe fn hsum_epi16_sse3(v: __m128i) -> u16 {
    let sum = _mm_hadd_epi16(v, _mm_set1_epi8(0));
    let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
    let sum = _mm_hadd_epi16(sum, _mm_set1_epi8(0));
    _mm_extract_epi16::<0>(sum) as u16
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CPCoord(pub u16);

impl From<CornerCubieCube> for CPCoord {
    fn from(value: CornerCubieCube) -> Self {
        unsafe {
            let cp_values = _mm_and_si128(_mm_srli_epi32::<5>(value.0), _mm_set1_epi8(0b111));

            //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
            let values_67 = _mm_shuffle_epi8(cp_values, _mm_set_epi8  (-1, -1, 7, -1, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6));
            let values_2345 = _mm_shuffle_epi8(cp_values, _mm_set_epi8(5, 4, -1, -1, 5, 4, 3, -1, 5, 4, 3, 2, 5, 4, 3, 2));
            let values_15 = _mm_shuffle_epi8(cp_values, _mm_set_epi64x(5, 1));

            let higher_left_67 = _mm_and_si128(_mm_cmplt_epi8(values_67, _mm_shuffle_epi8(cp_values, _mm_set_epi8(
                -1, -1, 6, -1, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0
            ))), _mm_set1_epi8(1));
            let higher_left_2345 = _mm_and_si128(_mm_cmplt_epi8(values_2345, _mm_shuffle_epi8(cp_values, _mm_set_epi8(
                3, 3, -1, -1, 2, 2, 2, -1, 1, 1, 1, 1, 0, 0, 0, 0
            ))), _mm_set1_epi8(1));
            let higher_left_15 = _mm_and_si128(_mm_cmplt_epi8(values_15, _mm_shuffle_epi8(cp_values, _mm_set_epi64x(4, 0))), _mm_set1_epi8(1));

            let hsum = _mm_hadd_epi32(higher_left_2345, higher_left_67);
            //7676767654325432
            let hsum = _mm_hadd_epi32(hsum, higher_left_15);
            //0005000176765432
            let hsum = _mm_shuffle_epi8(hsum, _mm_set_epi8(-1, 7, -1, 5, 6, 12, 4, 3, -1, -1, 2, 1, -1, -1, 0, 8));
            //0707656500430021
            let hsum = _mm_hadd_epi16(hsum, _mm_set1_epi8(0));
            //0000000007654321
            let hsum = _mm_shuffle_epi8(hsum, _mm_set_epi8(-1, -1, -1, 6, -1, 5, -1, 4, -1, 3, -1, 2, -1, 1, -1, 0));
            //0007060504030201
            let factorials = _mm_set_epi16(0, 5040, 720, 120, 24, 6, 2, 1);
            let prod = _mm_mullo_epi16(hsum, factorials);

            CPCoord(hsum_epi16_sse3(prod))
        }
    }
}

impl From<[Corner; 8]> for CPCoord {
    fn from(value: [Corner; 8]) -> Self {
        let mut cp = 0_u16;
        let factorial = [1, 2, 6, 24, 120, 720, 5040];

        for i in 1..8 {
            let mut higher = 0;
            for j in 0..i {
                if value[i].id < value[j].id {
                    higher += 1;
                }
            }
            cp += factorial[i - 1] * higher;
        }
        CPCoord(cp)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EPCoord(pub u32);

impl From<EdgeCubieCube> for EPCoord {
    fn from(value: EdgeCubieCube) -> Self {
        unsafe {
            let ep_values_m128 = _mm_and_si128(_mm_srli_epi32::<4>(value.0), _mm_set1_epi8(0b1111));
            let ep_values = _mm256_setr_m128i(ep_values_m128, ep_values_m128);

            let values_1011 = _mm256_shuffle_epi8(ep_values, _mm256_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 11, -1, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10, 11, 10));
            let values_6789 = _mm256_shuffle_epi8(ep_values, _mm256_set_epi8(9, 8, -1, -1, 9, 8, 7, -1, 9, 8, 7, 6, 9, 8, 7, 6, 9, 8, 7, 6, 9, 8, 7, 6, 9, 8, 7, 6, 9, 8, 7, 6));
            let values_2345 = _mm_shuffle_epi8(ep_values_m128, _mm_set_epi8(5, 4, 5, 4, 5, 4, 5, 4, -1, -1, 3, -1, 3, 2, 3, 2));

            let values_159 = _mm_shuffle_epi8(ep_values_m128, _mm_set_epi8(-1, -1, -1, -1, 9, -1, -1, -1, 1, -1, -1, 1, 5, -1, -1, -1));

            let higher_left_1011 = _mm256_and_si256(_mm256_cmpgt_epi8(_mm256_shuffle_epi8(ep_values, _mm256_set_epi8(
                -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 10, -1, 9, 9, 8, 8,
                7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0,
            )), values_1011), _mm256_set1_epi8(1));

            let higher_left_6789 = _mm256_and_si256(_mm256_cmpgt_epi8(_mm256_shuffle_epi8(ep_values, _mm256_set_epi8(
                7, 7, -1, -1, 6, 6, 6, -1, 5, 5, 5, 5, 4, 4, 4, 4,
                3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1, 1, 0, 0, 0, 0,
            )), values_6789), _mm256_set1_epi8(1));

            let higher_left_2345 = _mm_and_si128(_mm_cmplt_epi8(values_2345, _mm_shuffle_epi8(ep_values_m128, _mm_set_epi8(
                3, 3, 2, 2, 1, 1, 0, 0, -1, -1, 2, -1, 1, 1, 0, 0
            ))), _mm_set1_epi8(1));

            let higher_left_159 = _mm_and_si128(_mm_cmpgt_epi8(_mm_shuffle_epi8(ep_values_m128, _mm_set_epi8(
                -1, -1, -1, -1, 8, -1, -1, -1, -1, -1, -1, 0, 4, -1, -1, -1
            )), values_159), _mm_set1_epi8(1));

            let hsum = _mm256_hadd_epi32(higher_left_6789, higher_left_1011);

            let hsum_lo = _mm256_castsi256_si128(hsum);
            let hsum_hi = _mm256_extracti128_si256::<1>(hsum);
            let hsum = _mm_add_epi8(hsum_lo, hsum_hi);

            let hsum_with_6789 = _mm_hadd_epi32(higher_left_2345, hsum);
            let hsum = _mm_and_si128(_mm_hadd_epi16(hsum_with_6789, _mm_set1_epi8(0)), _mm_set_epi16(0, 0, 0, 0, -1, 0, -1, -1));
            let hsum = _mm_add_epi8(hsum, _mm_and_si128(hsum_with_6789, _mm_set_epi16(0, 0, -1, -1, 0, 0, 0, 0)));
            let hsum = _mm_add_epi8(hsum, higher_left_159);

            let hsum_lower_factorials = _mm_shuffle_epi8(hsum, _mm_set_epi8(
                -1, -1, -1, -1, -1, -1, -1, 1, -1, -1, -1, 0, -1, -1, -1, 4,
            ));

            let prod1 = _mm_mullo_epi32(hsum_lower_factorials, _mm_set_epi32(0, 6, 2, 1));

            let hsum_higher_factorials = _mm256_shuffle_epi8(_mm256_setr_m128i(hsum, hsum), _mm256_set_epi8(
                -1, -1, -1, 7, -1, -1, -1, 6, -1, -1, -1, 11, -1, -1, -1, 10,
                -1, -1, -1, 9, -1, -1, -1, 8, -1, -1, -1, 3, -1, -1, -1, 2,
            ));

            let prod2 = _mm256_mullo_epi32(hsum_higher_factorials, _mm256_set_epi32(39916800, 3628800, 362880, 40320, 5040, 720, 120, 24));

            let prodsum = _mm256_hadd_epi32(prod2, _mm256_set1_epi64x(0));

            let prodsum_lo = _mm256_castsi256_si128(prodsum);
            let prodsum_hi = _mm256_extracti128_si256::<1>(prodsum);
            let prodsum = _mm_add_epi32(prodsum_lo, prodsum_hi);

            let prodsum = _mm_add_epi32(prodsum, _mm_hadd_epi32(prod1, _mm_set1_epi8(0)));
            let prodsum = _mm_hadd_epi32(prodsum, _mm_set1_epi8(0));

            EPCoord(_mm_extract_epi32::<0>(prodsum) as u32)
        }
    }
}

impl From<[Edge; 12]> for EPCoord {
    fn from(value: [Edge; 12]) -> Self {
        let mut ep = 0_u32;
        let factorial = [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];

        for i in 1..12 {
            let mut higher = 0;
            for j in 0..i {
                if value[i].id < value[j].id {
                    higher += 1;
                }
            }
            ep += factorial[i - 1] * higher;
        }
        EPCoord(ep)
    }
}