#[cfg(target_feature = "avx2")]
pub mod avx2_coord {
    use std::arch::x86_64::{__m128, __m128i, _mm256_add_epi32, _mm256_and_si256, _mm256_broadcastsi128_si256, _mm256_castsi128_si256, _mm256_castsi256_si128, _mm256_cmpgt_epi8, _mm256_extractf128_ps, _mm256_extracti128_si256, _mm256_hadd_epi16, _mm256_hadd_epi32, _mm256_mul_epi32, _mm256_mullo_epi32, _mm256_or_si256, _mm256_set1_epi64x, _mm256_set1_epi8, _mm256_set_epi32, _mm256_set_epi64x, _mm256_set_epi8, _mm256_setr_epi8, _mm256_setr_m128, _mm256_setr_m128i, _mm256_shuffle_epi32, _mm256_shuffle_epi8, _mm_add_epi32, _mm_add_epi64, _mm_add_epi8, _mm_add_ss, _mm_and_si128, _mm_castpd_si128, _mm_castps_si128, _mm_castsi128_ps, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_cmplt_epi8, _mm_cvtss_si32, _mm_extract_epi16, _mm_extract_epi32, _mm_hadd_epi16, _mm_hadd_epi32, _mm_hsub_epi16, _mm_insert_epi8, _mm_movehdup_ps, _mm_movehl_ps, _mm_movemask_epi8, _mm_mullo_epi16, _mm_mullo_epi32, _mm_or_si128, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_shufflelo_epi16, _mm_sll_epi32, _mm_slli_epi64, _mm_slli_si128, _mm_srli_epi32, _mm_sub_epi8};
    use std::cmp::Ordering;

    use crate::alignment::avx2::C;
    use crate::coord::{COUDCoord, CPCoord, EOCoordAll, EOCoordFB, EOCoordLR, EOCoordNoUDSlice, EOCoordUD, EPCoord, UDSliceUnsortedCoord};
    use crate::cube::{Corner, Edge};
    use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_all(value: &EdgeCubieCube) -> EOCoordAll {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(value.0, _mm_set_epi8(0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F));
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        let lr = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordAll(EOCoordUD(ud), EOCoordFB(fb), EOCoordLR(lr))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate)  unsafe fn unsafe_from_eocoord_ud(value: &EdgeCubieCube) -> EOCoordUD {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(value.0, _mm_set_epi8(0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F));
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        EOCoordUD(ud)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate)  unsafe fn unsafe_from_eocoord_fb(value: &EdgeCubieCube) -> EOCoordFB {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(value.0, _mm_set_epi8(0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F));
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        EOCoordFB(fb)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate)  unsafe fn unsafe_from_eocoord_lr(value: &EdgeCubieCube) -> EOCoordLR {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(value.0, _mm_set_epi8(0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F));
        let rl = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordLR(rl)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_no_ud_slice(value: &EdgeCubieCube) -> EOCoordNoUDSlice {
        let no_slice_edges_no_db = _mm_shuffle_epi8(value.0, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, 10, 9, 8, 3, 2, 1, 0));
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_slice_edges_no_db)) as u8;
        EOCoordNoUDSlice(ud)
    }

    const CO_MUL: __m128i = unsafe { C { a_u16: [1, 3, 9, 27, 81, 243, 729, 0] }.a };
    const CO_SHUFFLE_8_TO_16: __m128i = unsafe { C { a_u8: [0, 0xFF, 1, 0xFF, 2, 0xFF, 3, 0xFF, 4, 0xFF, 5, 0xFF, 6, 0xFF, 7, 0xFF] }.a };

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cocoord(value: &CornerCubieCube) -> COUDCoord {
        //Spread co data out into 16bit values to avoid overflow later
        let co_epi16 = _mm_and_si128(_mm_shuffle_epi8(value.0, CO_SHUFFLE_8_TO_16), _mm_set1_epi8(0b11));
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

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cpcoord(value: &CornerCubieCube) -> CPCoord {
        let cp_values = _mm_and_si128(_mm_srli_epi32::<5>(value.0), _mm_set1_epi8(0b111));

        //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
        let values_67 = _mm_shuffle_epi8(cp_values, _mm_set_epi8(-1, -1, 7, -1, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6));
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
        let hsum = _mm_hadd_epi32(hsum, higher_left_15);
        let hsum = _mm_shuffle_epi8(hsum, _mm_set_epi8(-1, 7, -1, 5, 6, 12, 4, 3, -1, -1, 2, 1, -1, -1, 0, 8));
        let hsum = _mm_hadd_epi16(hsum, _mm_set1_epi8(0));
        let hsum = _mm_shuffle_epi8(hsum, _mm_set_epi8(-1, -1, -1, 6, -1, 5, -1, 4, -1, 3, -1, 2, -1, 1, -1, 0));
        let factorials = _mm_set_epi16(0, 5040, 720, 120, 24, 6, 2, 1);
        let prod = _mm_mullo_epi16(hsum, factorials);

        CPCoord(hsum_epi16_sse3(prod))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_epcoord(value: &EdgeCubieCube) -> EPCoord {
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

    const UD_SLICE_BINOM_0_ARR: [u8; 16] = [b(0, 0), b(0, 1), b(0, 2), b(0, 3), b(1, 0), b(1, 1), b(1, 2), b(1, 3), b(2, 0), b(2, 1), b(2, 2), b(2, 3), b(3, 0), b(3, 1), b(3, 2), b(3, 3)];
    const UD_SLICE_BINOM_1_ARR: [u8; 16] = [b(4, 0), b(4, 1), b(4, 2), b(4, 3), b(5, 0), b(5, 1), b(5, 2), b(5, 3), b(6, 0), b(6, 1), b(6, 2), b(6, 3), b(7, 0), b(7, 1), b(7, 2), b(7, 3)];
    const UD_SLICE_BINOM_2_ARR: [u8; 16] = [b(8, 0), b(8, 1), b(8, 2), b(8, 3), b(9, 0), b(9, 1), b(9, 2), b(9, 3), b(10, 0), b(10, 1), b(10, 2), b(10, 3), b(11, 0), b(11, 1), b(11, 2), b(11, 3)];

    const UD_SLICE_BINOM_0: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_0_ARR }.a };
    const UD_SLICE_BINOM_1: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_1_ARR }.a };
    const UD_SLICE_BINOM_2: __m128i = unsafe { C { a_u8: UD_SLICE_BINOM_2_ARR }.a };


    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_udsliceunsortedcoord(value: &EdgeCubieCube) -> UDSliceUnsortedCoord {
        let coord = unsafe {
            let slice_edges = _mm_srli_epi32::<6>(_mm_and_si128(value.0, _mm_set1_epi8(0b01000000)));
            //Our edge order is
            // UB UR UF UL FR FL BR BL DF DR DB DL

            //Kociemba uses
            // UR UF UL UB DR DF DL DB FR FL BL BR

            //We map to Kociemba's order here to make things simpler for us, but this could be optimized out if we just adjust the later shuffle masks
            let slice_edges = _mm_shuffle_epi8(slice_edges, _mm_set_epi8(-1, -1, -1, -1, 6, 7, 5, 4, 10, 11, 8, 9, 0, 3, 2, 1));

            let non_slice_edge_mask = _mm_cmpeq_epi8(slice_edges, _mm_set1_epi8(0));

            let e0123 = _mm_shuffle_epi8(slice_edges, _mm_set_epi8(3, -1, -1, -1, 2, 2, -1, -1, 1, 1, 1, -1, 0, 0, 0, 0));
            let e4567 = _mm_shuffle_epi8(slice_edges, _mm_set_epi8(7, -1, -1, -1, 6, 6, -1, -1, 5, 5, 5, -1, 4, 4, 4, 4));
            let e891011 = _mm_shuffle_epi8(slice_edges, _mm_set_epi8(11, -1, -1, -1, 10, 10, -1, -1, 9, 9, 9, -1, 8, 8, 8, 8));

            let hadd = _mm_hadd_epi32(e0123, e4567);
            let hadd = _mm_hadd_epi32(hadd, e891011);
            let hadd0123 = _mm_and_si128(hadd, _mm_set_epi32(0, 0, 0, -1));

            let hadd4567891011 = _mm_hadd_epi32(_mm_shuffle_epi8(hadd, _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 3, 3, 3)), _mm_set1_epi8(0));
            let hadd4567891011 = _mm_add_epi8(hadd4567891011, _mm_shuffle_epi8(hadd4567891011, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, 15, 15, 15, 15)));
            let hadd = _mm_or_si128(_mm_slli_si128::<4>(hadd4567891011), hadd0123);

            let hadd = _mm_and_si128(hadd, non_slice_edge_mask);

            let lut_index = _mm_and_si128(_mm_sub_epi8(hadd, _mm_set1_epi8(1)), _mm_set1_epi8(0b10001111_u8 as i8));
            let lut_index = _mm_add_epi8(lut_index, _mm_set_epi8(0, 0, 0, 0, 12, 8, 4, 0, 12, 8, 4, 0, 12, 8, 4, 0));

            let binom0123 = _mm_and_si128(_mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index), _mm_set_epi32(0, 0, 0, -1));
            let binom4567 = _mm_and_si128(_mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index), _mm_set_epi32(0, 0, -1, 0));
            let binom891011 = _mm_and_si128(_mm_shuffle_epi8(UD_SLICE_BINOM_2, lut_index), _mm_set_epi32(0, -1, 0, 0));

            let hsum = _mm_or_si128(binom0123, _mm_or_si128(binom4567, binom891011));

            let hsum_u16 = _mm_sad_epu8(hsum, _mm_set1_epi8(0));

            let hsum = _mm_hadd_epi32(_mm_shuffle_epi32::<0b11111000>(hsum_u16), _mm_set1_epi32(0));

            _mm_extract_epi16::<0>(hsum) as u16
        };
        UDSliceUnsortedCoord(coord)
    }

    const FACTORIAL: [u32; 12] = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];

    const fn b(n: u8, k: u8) -> u8 {
        if n == 0 || n < k {
            return 0;
        }
        (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
    }
}