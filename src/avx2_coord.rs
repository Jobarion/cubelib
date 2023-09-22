#[cfg(target_feature = "avx2")]
pub mod avx2_coord {
    use std::arch::x86_64::{__m128i, _mm256_and_si256, _mm256_castsi256_si128, _mm256_cmpgt_epi8, _mm256_extracti128_si256, _mm256_hadd_epi32, _mm256_mullo_epi32, _mm256_set1_epi64x, _mm256_set1_epi8, _mm256_set_epi32, _mm256_set_epi8, _mm256_setr_m128i, _mm256_shuffle_epi8, _mm_add_epi32, _mm_add_epi8, _mm_and_si128, _mm_castps_si128, _mm_castsi128_ps, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_cmplt_epi8, _mm_extract_epi16, _mm_extract_epi32, _mm_extract_epi64, _mm_extract_epi8, _mm_hadd_epi16, _mm_hadd_epi32, _mm_movemask_epi8, _mm_mullo_epi16, _mm_mullo_epi32, _mm_or_si128, _mm_permute_pd, _mm_permute_ps, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_sll_epi64, _mm_slli_epi32, _mm_slli_epi64, _mm_slli_si128, _mm_srli_epi32, _mm_sub_epi8, _mm_xor_si128};

    use crate::alignment::avx2::C;
    use crate::avx2_cubie::avx2_cubie;
    use crate::coord::{COUDCoord, CPCoord, CPOrbitTwistParityCoord, CPOrbitUnsortedCoord, EOCoordAll, EOCoordFB, EOCoordLR, EOCoordNoUDSlice, EOCoordUD, EPCoord, FBSliceUnsortedCoord, ParityCoord, UDSliceUnsortedCoord};
    use crate::cube::{ApplyAlgorithm, Axis, NewSolved, Turnable};
    use crate::cube::Turn::{Clockwise, CounterClockwise, Half};
    use crate::cubie::{CornerCubieCube, EdgeCubieCube};
    use crate::htr::HTR_MOVES;

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
    pub(crate) unsafe fn unsafe_from_udslice_unsorted_coord(value: &EdgeCubieCube) -> UDSliceUnsortedCoord {
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

    const ORBIT_STATE_LUT: [u8; 56] = [
    //  x, F, L, U, U, L, F, x
        3, 3, 3, 3, 3, 3, 3, 3,  //x
        3, 0, 2, 1, 1, 2, 0, 3,  //F
        3, 1, 0, 2, 2, 0, 1, 3,  //L
        3, 2, 1, 0, 0, 1, 2, 3,  //U
        3, 2, 1, 0, 0, 1, 2, 3,  //U
        3, 1, 0, 2, 2, 0, 1, 3,  //L
        3, 0, 2, 1, 1, 2, 0, 3,  //F
     // 3, 3, 3, 3, 3, 3, 3, 3,  //x
    ];

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_cp_orbit_twist_parity_coord(cube: &CornerCubieCube) -> CPOrbitTwistParityCoord {
        let orbit_corners = arrange_orbit_corners(cube.0);

        let relevant_corners = _mm_shuffle_epi8(orbit_corners, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 5, 4, 2, 1, 0));

        let ud = _mm_movemask_epi8(relevant_corners);
        // let fb = _mm_movemask_epi8(_mm_add_epi8(relevant_corners, _mm_set1_epi8(0b01000000)));
        // let lr = _mm_movemask_epi8(_mm_slli_epi32::<1>(_mm_add_epi8(relevant_corners, _mm_set1_epi8(0b00100000))));

        // println!("{:?}", ud);
        // println!("{:?}", fb);
        // println!("{:?}", lr);

        let ud_twist = ORBIT_STATE_LUT[ud as usize];
        // let fb_twist = ORBIT_STATE_LUT[fb as usize];
        // let lr_twist = ORBIT_STATE_LUT[lr as usize];

        CPOrbitTwistParityCoord(ud_twist)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_parity_coord(cube: &CornerCubieCube) -> ParityCoord {
        let values_12345 = _mm_shuffle_epi8(cube.0, _mm_set_epi8(-1, 5, 5, 5, 5, 5, 4, 4, 4, 4, 3, 3, 3 , 2, 2, 1));
        let values_67 = _mm_shuffle_epi8(cube.0, _mm_set_epi8(-1, -1, -1, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6));

        let higher_left_12345 = _mm_and_si128(_mm_cmplt_epi8(values_12345, _mm_shuffle_epi8(cube.0, _mm_set_epi8(
            -1, 4, 3, 2, 1, 0, 3, 2, 1, 0, 2, 1, 0, 1, 0, 0
        ))), _mm_set1_epi8(1));

        let higher_left_67 = _mm_and_si128(_mm_cmplt_epi8(values_67, _mm_shuffle_epi8(cube.0, _mm_set_epi8(
            -1, -1, -1, 6, 5, 4, 3, 2, 1, 0, 5, 4, 3, 2, 1, 0
        ))), _mm_set1_epi8(1));

        let parity = _mm_xor_si128(higher_left_12345, higher_left_67);
        let parity = _mm_sad_epu8(parity, _mm_set1_epi8(0));
        let parity = _mm_extract_epi64::<0>(_mm_castps_si128(_mm_permute_ps::<0b00001000>(_mm_castsi128_ps(parity))));
        let parity = (parity ^ (parity >> 32)) & 1;

        ParityCoord(parity == 1)
    }

    // #[target_feature(enable = "avx2")]
    // #[inline]
    // pub unsafe fn unsafe_from_parity_coord(cube: &CornerCubieCube) -> ParityCoord {
    //     let pairwise_swaps = _mm_and_si128(_mm_cmplt_epi8(cube.0, _mm_srli_epi32::<32>(cube.0)), _mm_set1_epi8(1));
    //     let coord = _mm_extract_epi8::<0>(_mm_sad_epu8(pairwise_swaps, _mm_set1_epi8(0))) & 1;
    //
    //     ParityCoord(coord == 1)
    // }

    const CP_ORBIT_SHUFFLE_BLOCK_0: [__m128i; 16] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0000
        unsafe { C { a_u8: [1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0001
        unsafe { C { a_u8: [0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0010
        unsafe { C { a_u8: [2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0011
        unsafe { C { a_u8: [0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0100
        unsafe { C { a_u8: [1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0101
        unsafe { C { a_u8: [0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0110
        unsafe { C { a_u8: [3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0111
        unsafe { C { a_u8: [0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1000
        unsafe { C { a_u8: [1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1001
        unsafe { C { a_u8: [0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1010
        unsafe { C { a_u8: [2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1011
        unsafe { C { a_u8: [0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1100
        unsafe { C { a_u8: [1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1101
        unsafe { C { a_u8: [0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1110
        unsafe { C { a_u8: [0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//1111
    ];

    const CP_ORBIT_SHUFFLE_BLOCK_1: [__m128i; 16] = [
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },//0000
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF] }.a },//0001
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF] }.a },//0010
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF] }.a },//0011
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF] }.a },//0100
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF] }.a },//0101
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF] }.a },//0110
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF] }.a },//0111
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF] }.a },//1000
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF] }.a },//1001
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF] }.a },//1010
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF] }.a },//1011
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF] }.a },//1100
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF] }.a },//1101
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF] }.a },//1110
        unsafe { C { a_u8: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7] }.a },//1111
    ];

    const CP_ORBIT_SHUFFLE_GAP_0: [__m128i; 5] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 1, 2, 4, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 1, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [0, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
        unsafe { C { a_u8: [4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a },
    ];

    const CP_ORBIT_SHUFFLE_GAP_1: [__m128i; 5] = [
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 10, 12, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 9, 12, 13, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 8, 12, 13, 14, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 12, 13, 14, 15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
    ];

    unsafe fn arrange_orbit_corners(value: __m128i) -> __m128i {
        let corners_with_marker = _mm_or_si128(value, _mm_set_epi8(-1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0));
        let ud_corners = _mm_movemask_epi8(_mm_slli_epi32::<2>(value)) as usize;
        let block_0 = ud_corners & 0xF;
        let block_1 = (ud_corners >> 4) & 0xF;

        let ud_corners_sorted_gaps = _mm_or_si128(_mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_0[block_0]), _mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_1[block_1]));

        let gaps = _mm_and_si128(_mm_cmpeq_epi8(ud_corners_sorted_gaps, _mm_set1_epi8(-1)), _mm_set1_epi8(1));
        let gap_sizes = _mm_sad_epu8(gaps, _mm_set1_epi8(0));

        let gap_sizes = _mm_extract_epi64::<0>(_mm_shuffle_epi32::<0b11111000>(gap_sizes)) as usize;
        let gap_0 = gap_sizes & 0xF;
        let gap_1 = (gap_sizes >> 32) & 0xF;

        _mm_shuffle_epi8(_mm_shuffle_epi8(ud_corners_sorted_gaps, CP_ORBIT_SHUFFLE_GAP_0[gap_0]), CP_ORBIT_SHUFFLE_GAP_1[gap_1])
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_fbslice_unsorted_coord(value: &EdgeCubieCube) -> FBSliceUnsortedCoord {
        let fb_slice_edges = _mm_shuffle_epi8(
            _mm_set_epi8(0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0),
            _mm_and_si128(_mm_srli_epi32::<4>(value.0), _mm_set1_epi8(0x0F))
        );
        let fb_slice_edges = _mm_shuffle_epi8(fb_slice_edges, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 11, 9, 3, 1, 10, 8, 2, 0));

        FBSliceUnsortedCoord(unsorted_coord_4_4_split(fb_slice_edges))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cp_orbit_unsorted_coord(value: &CornerCubieCube) -> CPOrbitUnsortedCoord {
        let orbit_corners = _mm_srli_epi32::<5>(_mm_and_si128(value.0, _mm_set1_epi8(0b00100000)));
        CPOrbitUnsortedCoord(unsorted_coord_4_4_split(orbit_corners))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsorted_coord_4_4_split(value: __m128i) -> u8 {
        let marked = value;
        let unmarked = _mm_cmpeq_epi8(marked, _mm_set1_epi8(0));

        let c0123 = _mm_shuffle_epi8(marked, _mm_set_epi8(3, -1, -1, -1, 2, 2, -1, -1, 1, 1, 1, -1, 0, 0, 0, 0));
        let c4567 = _mm_shuffle_epi8(marked, _mm_set_epi8(7, -1, -1, -1, 6, 6, -1, -1, 5, 5, 5, -1, 4, 4, 4, 4));

        let hadd = _mm_hadd_epi32(c0123, c4567);
        let hadd = _mm_hadd_epi32(hadd, _mm_set1_epi8(0));
        let hadd = _mm_add_epi8(hadd, _mm_shuffle_epi8(hadd, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 3, 3, 3, 3, -1, -1, -1, -1)));
        let hadd = _mm_and_si128(hadd, unmarked);

        let lut_index = _mm_and_si128(_mm_sub_epi8(hadd, _mm_set1_epi8(1)), _mm_set1_epi8(0b10001111_u8 as i8));
        let lut_index = _mm_add_epi8(lut_index, _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 12, 8, 4, 0, 12, 8, 4, 0));

        let binom0123 = _mm_and_si128(_mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index), _mm_set_epi32(0, 0, 0, -1));
        let binom4567 = _mm_and_si128(_mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index), _mm_set_epi32(0, 0, -1, 0));

        let sum = _mm_sad_epu8(_mm_or_si128(binom0123, binom4567), _mm_set1_epi8(0));

        _mm_extract_epi16::<0>(sum) as u8
    }

    const FACTORIAL: [u32; 12] = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];

    const fn b(n: u8, k: u8) -> u8 {
        if n == 0 || n < k {
            return 0;
        }
        (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
    }
}