use crate::puzzles::c333::{CornerCube333, Cube333, EdgeCube333};
use crate::puzzles::cube::coords::CPCoord;
use crate::steps::coord::Coord;

//Coordinate representing the position of edges that belong into the FB slice, assuming the UD slice is already correct.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FBSliceUnsortedCoord(pub(crate) u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPOrbitUnsortedCoord(pub(crate) u8);

//Coordinate representing the twist state of HTR corner orbits
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CPOrbitTwistCoord(pub(crate) u8);

//Coordinate representing the parity state
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ParityCoord(pub(crate) bool);

//Assuming we already have UD-DR, represents the combination of CPOrbitUnsortedCoord, CPOrbitTwistCoord and FBSliceUnsortedCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PureHTRDRUDCoord(pub(crate) u16);

//Assuming we already have UD-DR, represents the combination of CPCoord and FBSliceUnsortedCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ImpureHTRDRUDCoord(pub(crate) u32);

impl Coord<70> for FBSliceUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FBSliceUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<70> for CPOrbitUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPOrbitUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<6> for CPOrbitTwistCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPOrbitTwistCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2> for ParityCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for ParityCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

//TODO this should use 'impl const' once it's stable
pub const PURE_HTRDRUD_SIZE: usize = 70 * 70 * 6;
impl Coord<PURE_HTRDRUD_SIZE> for PureHTRDRUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for PureHTRDRUDCoord {
    fn into(self) -> usize {
        self.val()
    }
}

//TODO this should use 'impl const' once it's stable
pub const IMPURE_HTRDRUD_SIZE: usize = 70 * 40320;
impl Coord<{ IMPURE_HTRDRUD_SIZE }> for ImpureHTRDRUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for ImpureHTRDRUDCoord {
    fn into(self) -> usize {
        self.val()
    }
}

pub type HTRDRUDCoord = PureHTRDRUDCoord;
pub const HTRDRUD_SIZE: usize = PURE_HTRDRUD_SIZE;

impl From<&EdgeCube333> for FBSliceUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_fbslice_unsorted_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCube333) -> Self {
        wasm32::from_fbslice_unsorted_coord(value)
    }
}

impl From<&CornerCube333> for CPOrbitUnsortedCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCube333) -> Self {
        unsafe { avx2::unsafe_from_cp_orbit_unsorted_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CornerCube333) -> Self {
        wasm32::from_cp_orbit_unsorted_coord(value)
    }
}

impl From<&CornerCube333> for CPOrbitTwistCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCube333) -> Self {
        unsafe { avx2::unsafe_from_cp_orbit_twist_parity_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CornerCube333) -> Self {
        wasm32::from_cp_orbit_twist_parity_coord(value)
    }
}

impl From<&CornerCube333> for ParityCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCube333) -> Self {
        unsafe { avx2::unsafe_from_parity_coord(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &CornerCube333) -> Self {
        wasm32::from_parity_coord(value)
    }
}

impl From<&Cube333> for PureHTRDRUDCoord {
    fn from(value: &Cube333) -> Self {
        let ep_fbslice_coord = FBSliceUnsortedCoord::from(&value.edges).val();
        let cp_orbit_coord = CPOrbitUnsortedCoord::from(&value.corners).val();
        let cp_orbit_twist = CPOrbitTwistCoord::from(&value.corners).val();

        let val = cp_orbit_twist
            + cp_orbit_coord * CPOrbitTwistCoord::size()
            + ep_fbslice_coord * CPOrbitTwistCoord::size() * CPOrbitUnsortedCoord::size();
        Self(val as u16)
    }
}

impl From<&Cube333> for ImpureHTRDRUDCoord {
    fn from(value: &Cube333) -> Self {
        let ep_fbslice_coord = FBSliceUnsortedCoord::from(&value.edges).val();
        let cp = CPCoord::from(&value.corners).val();

        let val = cp + ep_fbslice_coord * CPCoord::size();
        Self(val as u32)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_castps_si128, _mm_castsi128_ps, _mm_cmpeq_epi8, _mm_cmplt_epi8, _mm_extract_epi16, _mm_extract_epi64, _mm_hadd_epi16, _mm_hadd_epi32, _mm_movemask_epi8, _mm_or_si128, _mm_permute_ps, _mm_sad_epu8, _mm_set1_epi8, _mm_set_epi16, _mm_setr_epi16, _mm_setr_epi32, _mm_setr_epi8, _mm_shuffle_epi32, _mm_shuffle_epi8, _mm_sll_epi16, _mm_slli_epi16, _mm_slli_epi32, _mm_sra_epi16, _mm_sra_epi32, _mm_srli_epi32, _mm_srli_epi64, _mm_sub_epi8, _mm_xor_si128};

    use crate::alignment::avx2::C;
    use crate::puzzles::c333::{CornerCube333, EdgeCube333};
    use crate::puzzles::c333::steps::htr::coords::{CPOrbitTwistCoord, CPOrbitUnsortedCoord, FBSliceUnsortedCoord, ParityCoord};

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

    const UD_SLICE_BINOM_0: __m128i = unsafe {
        C { a_u8: UD_SLICE_BINOM_0_ARR, }.a
    };
    const UD_SLICE_BINOM_1: __m128i = unsafe {
        C { a_u8: UD_SLICE_BINOM_1_ARR, }.a
    };

    const ORBIT_STATE_LUT: [u8; 56] = [
        //  x, F, L, U, U, L, F, x
        3, 3, 3, 3, 3, 3, 3, 3, //x
        3, 0, 2, 1, 1, 2, 0, 3, //F
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 0, 2, 1, 1, 2, 0, 3, //F
        // 3, 3, 3, 3, 3, 3, 3, 3,  //x
    ];



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
        let corners_with_marker = _mm_or_si128(
            value,
            _mm_setr_epi8( 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,-1),
        );
        let ud_corners = _mm_movemask_epi8(_mm_slli_epi32::<2>(value)) as usize;
        let block_0 = ud_corners & 0xF;
        let block_1 = (ud_corners >> 4) & 0xF;

        let ud_corners_sorted_gaps = _mm_or_si128(
            _mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_0[block_0]),
            _mm_shuffle_epi8(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_1[block_1]),
        );

        let gaps = _mm_and_si128(
            _mm_cmpeq_epi8(ud_corners_sorted_gaps, _mm_set1_epi8(-1)),
            _mm_set1_epi8(1),
        );
        let gap_sizes = _mm_sad_epu8(gaps, _mm_set1_epi8(0));

        let gap_sizes = _mm_extract_epi64::<0>(_mm_shuffle_epi32::<0b11111000>(gap_sizes)) as u64;
        let gap_0 = gap_sizes & 0xF;
        let gap_1 = (gap_sizes >> 32) & 0xF;

        _mm_shuffle_epi8(
            _mm_shuffle_epi8(ud_corners_sorted_gaps, CP_ORBIT_SHUFFLE_GAP_0[gap_0 as usize]),
            CP_ORBIT_SHUFFLE_GAP_1[gap_1 as usize],
        )
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_fbslice_unsorted_coord(
        value: &EdgeCube333,
    ) -> FBSliceUnsortedCoord {
        let fb_slice_edges = _mm_shuffle_epi8(
            _mm_setr_epi8( 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0,0),
            _mm_and_si128(_mm_srli_epi32::<4>(value.0), _mm_set1_epi8(0x0F)),
        );
        let fb_slice_edges = _mm_shuffle_epi8(
            fb_slice_edges,
            _mm_setr_epi8( 0, 2, 8, 10, 1, 3, 9, 11, -1, -1, -1, -1, -1, -1, -1,-1),
        );

        FBSliceUnsortedCoord(unsorted_coord_4_4_split(fb_slice_edges))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_cp_orbit_unsorted_coord(
        value: &CornerCube333,
    ) -> CPOrbitUnsortedCoord {
        let orbit_corners = _mm_srli_epi32::<5>(_mm_and_si128(value.0, _mm_set1_epi8(0b00100000)));
        let orbit_corners = _mm_shuffle_epi8(
            orbit_corners,
            _mm_setr_epi8( 0, 2, 4, 6, 1, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1,-1),
        );
        CPOrbitUnsortedCoord(unsorted_coord_4_4_split(orbit_corners))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsorted_coord_4_4_split(value: __m128i) -> u8 {
        let marked = value;
        let unmarked = _mm_cmpeq_epi8(marked, _mm_set1_epi8(0));

        let c0123 = _mm_shuffle_epi8(
            marked,
            _mm_setr_epi8( 0, 0, 0, 0, -1, 1, 1, 1, -1, -1, 2, 2, -1, -1, -1,3),
        );
        let c4567 = _mm_shuffle_epi8(
            marked,
            _mm_setr_epi8( 4, 4, 4, 4, -1, 5, 5, 5, -1, -1, 6, 6, -1, -1, -1,7),
        );

        let hadd = _mm_hadd_epi32(c0123, c4567);
        let hadd = _mm_hadd_epi32(hadd, _mm_set1_epi8(0));
        let hadd = _mm_add_epi8(
            hadd,
            _mm_shuffle_epi8(
                hadd,
                _mm_setr_epi8( -1, -1, -1, -1, 3, 3, 3, 3, -1, -1, -1, -1, -1, -1, -1,-1),
            ),
        );
        let hadd = _mm_and_si128(hadd, unmarked);

        let lut_index = _mm_and_si128(
            _mm_sub_epi8(hadd, _mm_set1_epi8(1)),
            _mm_set1_epi8(0b10001111_u8 as i8),
        );
        let lut_index = _mm_add_epi8(
            lut_index,
            _mm_setr_epi8( 0, 4, 8, 12, 0, 4, 8, 12, 0, 0, 0, 0, 0, 0, 0,0),
        );

        let binom0123 = _mm_and_si128(
            _mm_shuffle_epi8(UD_SLICE_BINOM_0, lut_index),
            _mm_setr_epi32( -1, 0, 0,0),
        );
        let binom4567 = _mm_and_si128(
            _mm_shuffle_epi8(UD_SLICE_BINOM_1, lut_index),
            _mm_setr_epi32( 0, -1, 0,0),
        );

        let sum = _mm_sad_epu8(_mm_or_si128(binom0123, binom4567), _mm_set1_epi8(0));

        _mm_extract_epi16::<0>(sum) as u8
    }

    const CP_ORBIT_TWO_SWAP: [__m128i; 4] = [
        unsafe { C { a_u8: [3, 2, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [2, 3, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [1, 0, 3, 2, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
        unsafe { C { a_u8: [0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF] }.a },
    ];

    const ORBIT_TYPES: [u8; 10] = [0xFF, 1, 2, 0xFF, 0, 0xFF, 5, 0xFF, 3, 4];
    const CORNER_ID_ACUWVXBD_TRACING_MAP: __m128i = unsafe { C { a_u8: [0, 2, 4, 6, 5, 7, 1, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a };
    const CORNER_ID_ACUWVXBD_NUMBERING_MAP: __m128i = unsafe { C { a_u8: [0, 2, 1, 3, 2, 0, 3, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a };

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_cp_orbit_twist_parity_coord(
        cube: &CornerCube333,
    ) -> CPOrbitTwistCoord {
        // We need a point symmetrical tracing order for this to work
        let acuwvxbd = _mm_shuffle_epi8(cube.0, CORNER_ID_ACUWVXBD_TRACING_MAP);
        let orbit_corners = _mm_srli_epi64::<5>(arrange_orbit_corners(acuwvxbd));
        let orbit_corner_ids = _mm_shuffle_epi8(CORNER_ID_ACUWVXBD_NUMBERING_MAP, orbit_corners);

        let orbit_b = _mm_shuffle_epi8(orbit_corner_ids, _mm_setr_epi8(4, 5, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 ,-1 ,-1));
        let mut inverse_b = orbit_b;

        for _ in 0..10 {
            inverse_b = _mm_shuffle_epi8(inverse_b, orbit_b);
        }

        let perm_c = _mm_shuffle_epi8(inverse_b, orbit_corner_ids);
        let two_swap_mask = _mm_movemask_epi8(_mm_cmpeq_epi8(perm_c, _mm_set1_epi8(3)));
        let perm_c = _mm_shuffle_epi8(perm_c, CP_ORBIT_TWO_SWAP[(two_swap_mask.trailing_zeros() & 0b11) as usize]);

        let values = _mm_slli_epi16::<2>(perm_c);
        let values = _mm_srli_epi32::<8>(_mm_and_si128(values, _mm_setr_epi8(0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)));
        let values = _mm_or_si128(values, perm_c);

        let orbit_type = ORBIT_TYPES[_mm_extract_epi16::<0>(values) as usize & 0b1111];

        CPOrbitTwistCoord(orbit_type)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_parity_coord(cube: &CornerCube333) -> ParityCoord {
        let values_12345 = _mm_shuffle_epi8(
            cube.0,
            _mm_setr_epi8( 1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5,-1),
        );
        let values_67 = _mm_shuffle_epi8(
            cube.0,
            _mm_setr_epi8( 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, -1, -1,-1),
        );

        let higher_left_12345 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_12345,
                _mm_shuffle_epi8(
                    cube.0,
                    _mm_setr_epi8( 0, 0, 1, 0, 1, 2, 0, 1, 2, 3, 0, 1, 2, 3, 4,-1),
                ),
            ),
            _mm_set1_epi8(1),
        );

        let higher_left_67 = _mm_and_si128(
            _mm_cmplt_epi8(
                values_67,
                _mm_shuffle_epi8(
                    cube.0,
                    _mm_setr_epi8( 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, -1, -1,-1),
                ),
            ),
            _mm_set1_epi8(1),
        );

        let parity = _mm_xor_si128(higher_left_12345, higher_left_67);
        let parity = _mm_sad_epu8(parity, _mm_set1_epi8(0));
        let parity = _mm_extract_epi64::<0>(_mm_castps_si128(_mm_permute_ps::<0b00001000>(
            _mm_castsi128_ps(parity),
        )));
        let parity = (parity ^ (parity >> 32)) & 1;

        ParityCoord(parity == 1)
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
    use std::arch::wasm32::{i32x4, i8x16, u16x8, u16x8_extract_lane, u16x8_mul, u32x4_shl, u32x4_shr, u32x4_shuffle, u64x2, u64x2_extract_lane, u8x16, u8x16_add, u8x16_bitmask, u8x16_eq, u8x16_extract_lane, u8x16_lt, u8x16_sub, u8x16_swizzle, v128, v128_and, v128_or, v128_xor};

    use crate::puzzles::c333::{CornerCube333, EdgeCube333};
    use crate::puzzles::c333::steps::htr::coords::{CPCoord, CPOrbitTwistCoord, CPOrbitUnsortedCoord, FBSliceUnsortedCoord, ParityCoord};
    use crate::wasm_util::{hsum_narrow_epi16, hsum_narrow_epi32, hsum_wide_epi32, mm_sad_epu8, u8x16_set1};

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

    const ORBIT_STATE_LUT: [u8; 56] = [
        //  x, F, L, U, U, L, F, x
        3, 3, 3, 3, 3, 3, 3, 3, //x
        3, 0, 2, 1, 1, 2, 0, 3, //F
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 2, 1, 0, 0, 1, 2, 3, //U
        3, 1, 0, 2, 2, 0, 1, 3, //L
        3, 0, 2, 1, 1, 2, 0, 3, //F
        // 3, 3, 3, 3, 3, 3, 3, 3,  //x
    ];

    const CP_ORBIT_SHUFFLE_BLOCK_0: [v128; 16] = [
        u8x16(0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0000
        u8x16(1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0001
        u8x16(0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0010
        u8x16(2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0011
        u8x16(0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0100
        u8x16(1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0101
        u8x16(0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0110
        u8x16(3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//0111
        u8x16(0, 1, 2, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 3, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1000
        u8x16(1, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1001
        u8x16(0, 2, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1010
        u8x16(2, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1011
        u8x16(0, 1, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 2, 3, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1100
        u8x16(1, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1101
        u8x16(0, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2, 3, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF),//1110
        u8x16(0x0F, 0x0F, 0x0F, 0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF),//1111
    ];

    const CP_ORBIT_SHUFFLE_BLOCK_1: [v128; 16] = [
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),//0000
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF),//0001
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF),//0010
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF),//0011
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF),//0100
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF),//0101
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF),//0110
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF),//0111
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 7, 0xFF, 0xFF, 0xFF),//1000
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 7, 0xFF, 0xFF),//1001
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 7, 0xFF, 0xFF),//1010
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 7, 0xFF),//1011
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 6, 7, 0xFF, 0xFF),//1100
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 6, 7, 0xFF),//1101
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 5, 6, 7, 0xFF),//1110
        u8x16(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 4, 5, 6, 7),//1111
    ];

    const CP_ORBIT_SHUFFLE_GAP_0: [v128; 5] = [
        u8x16(0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15),
        u8x16(0, 1, 2, 4, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15),
        u8x16(0, 1, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15),
        u8x16(0, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15),
        u8x16(4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15),
    ];

    const CP_ORBIT_SHUFFLE_GAP_1: [v128; 5] = [
        u8x16(0, 1, 2, 3, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
        u8x16(0, 1, 2, 3, 8, 9, 10, 12, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
        u8x16(0, 1, 2, 3, 8, 9, 12, 13, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
        u8x16(0, 1, 2, 3, 8, 12, 13, 14, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
        u8x16(0, 1, 2, 3, 12, 13, 14, 15, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF),
    ];


    #[inline]
    pub(crate) fn from_fbslice_unsorted_coord(
        value: &EdgeCube333,
    ) -> FBSliceUnsortedCoord {
        let fb_slice_edges = u8x16_swizzle(
            u8x16( 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0,0),
            v128_and(u32x4_shr(value.0, 4), u8x16_set1(0x0F)),
        );
        let fb_slice_edges = u8x16_swizzle(
            fb_slice_edges,
            i8x16( 0, 2, 8, 10, 1, 3, 9, 11, -1, -1, -1, -1, -1, -1, -1,-1),
        );

        FBSliceUnsortedCoord(unsorted_coord_4_4_split(fb_slice_edges))
    }

    #[inline]
    fn unsorted_coord_4_4_split(value: v128) -> u8 {
        let marked = value;
        let unmarked = u8x16_eq(marked, u8x16_set1(0));

        let c0123 = u8x16_swizzle(
            marked,
            i8x16( 0, 0, 0, 0, -1, 1, 1, 1, -1, -1, 2, 2, -1, -1, -1,3),
        );
        let c4567 = u8x16_swizzle(
            marked,
            i8x16( 4, 4, 4, 4, -1, 5, 5, 5, -1, -1, 6, 6, -1, -1, -1,7),
        );

        let hadd = hsum_wide_epi32(c0123, c4567);
        let hadd = hsum_narrow_epi32(hadd);
        let hadd = u8x16_add(
            hadd,
            u8x16_swizzle(
                hadd,
                i8x16( -1, -1, -1, -1, 3, 3, 3, 3, -1, -1, -1, -1, -1, -1, -1,-1),
            ),
        );
        let hadd = v128_and(hadd, unmarked);

        let lut_index = v128_and(
            u8x16_sub(hadd, u8x16_set1(1)),
            u8x16_set1(0b10001111_u8),
        );
        let lut_index = u8x16_add(
            lut_index,
            u8x16( 0, 4, 8, 12, 0, 4, 8, 12, 0, 0, 0, 0, 0, 0, 0,0),
        );

        let binom0123 = v128_and(
            u8x16_swizzle(UD_SLICE_BINOM_0, lut_index),
            i32x4( -1, 0, 0,0),
        );
        let binom4567 = v128_and(
            u8x16_swizzle(UD_SLICE_BINOM_1, lut_index),
            i32x4( 0, -1, 0,0),
        );

        let sum = mm_sad_epu8(v128_or(binom0123, binom4567));
        u8x16_extract_lane::<0>(sum)
    }

    #[inline]
    fn hsum_epi16_sse3(v: v128) -> u16 {
        let sum = hsum_narrow_epi16(v);
        let sum = hsum_narrow_epi16(sum);
        let sum = hsum_narrow_epi16(sum);
        u16x8_extract_lane::<0>(sum)
    }

    #[inline]
    pub(crate) fn from_cpcoord(value: &CornerCube333) -> CPCoord {
        let cp_values = v128_and(u32x4_shr(value.0, 5), u8x16_set1(0b111));

        //We interleave the values to make using hadd_epi_<16/32> easier when we combine them
        let values_67 = u8x16_swizzle(
            cp_values,
            i8x16( 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, 6, 7, -1, 7, -1,-1),
        );
        let values_2345 = u8x16_swizzle(
            cp_values,
            i8x16( 2, 3, 4, 5, 2, 3, 4, 5, -1, 3, 4, 5, -1, -1, 4,5),
        );
        let values_15 = u8x16_swizzle(cp_values, u64x2(1, 5));

        let higher_left_67 = v128_and(
            u8x16_lt(
                values_67,
                u8x16_swizzle(
                    cp_values,
                    i8x16( 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, -1, 6, -1,-1),
                ),
            ),
            u8x16_set1(1),
        );
        let higher_left_2345 = v128_and(
            u8x16_lt(
                values_2345,
                u8x16_swizzle(
                    cp_values,
                    i8x16( 0, 0, 0, 0, 1, 1, 1, 1, -1, 2, 2, 2, -1, -1, 3,3),
                ),
            ),
            u8x16_set1(1),
        );
        let higher_left_15 = v128_and(
            u8x16_lt(values_15, u8x16_swizzle(cp_values, u64x2(0, 4))),
            u8x16_set1(1),
        );

        let hsum = hsum_wide_epi32(higher_left_2345, higher_left_67);
        let hsum = hsum_wide_epi32(hsum, higher_left_15);
        let hsum = u8x16_swizzle(
            hsum,
            i8x16( 8, 0, -1, -1, 1, 2, -1, -1, 3, 4, 12, 6, 5, -1, 7,-1),
        );
        let hsum = hsum_narrow_epi16(hsum);
        let hsum = u8x16_swizzle(
            hsum,
            i8x16( 0, -1, 1, -1, 2, -1, 3, -1, 4, -1, 5, -1, 6, -1, -1,-1),
        );
        let factorials = u16x8( 1, 2, 6, 24, 120, 720, 5040,0);
        let prod = u16x8_mul(hsum, factorials);

        CPCoord(hsum_epi16_sse3(prod))
    }

    #[inline]
    pub(crate) fn from_cp_orbit_unsorted_coord(
        value: &CornerCube333,
    ) -> CPOrbitUnsortedCoord {
        let orbit_corners = u32x4_shr(v128_and(value.0, u8x16_set1(0b00100000)), 5);
        let orbit_corners = u8x16_swizzle(
            orbit_corners,
            i8x16( 0, 2, 4, 6, 1, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1,-1),
        );
        CPOrbitUnsortedCoord(unsorted_coord_4_4_split(orbit_corners))
    }

    fn arrange_orbit_corners(value: v128) -> v128 {
        let corners_with_marker = v128_or(
            value,
            i8x16( 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,-1),
        );
        let ud_corners = u8x16_bitmask(u32x4_shl(value, 2)) as usize;
        let block_0 = ud_corners & 0xF;
        let block_1 = (ud_corners >> 4) & 0xF;

        let ud_corners_sorted_gaps = v128_or(
            u8x16_swizzle(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_0[block_0]),
            u8x16_swizzle(corners_with_marker, CP_ORBIT_SHUFFLE_BLOCK_1[block_1]),
        );

        let gaps = v128_and(
            u8x16_eq(ud_corners_sorted_gaps, u8x16_set1(0xFF)),
            u8x16_set1(1),
        );
        let gap_sizes = mm_sad_epu8(gaps);

        let gap_sizes = u64x2_extract_lane::<0>(u32x4_shuffle::<0, 2, 3, 3>(gap_sizes, u64x2(0, 0)));
        let gap_0 = gap_sizes & 0xF;
        let gap_1 = (gap_sizes >> 32) & 0xF;

        u8x16_swizzle(
            u8x16_swizzle(ud_corners_sorted_gaps, CP_ORBIT_SHUFFLE_GAP_0[gap_0 as usize]),
            CP_ORBIT_SHUFFLE_GAP_1[gap_1 as usize],
        )
    }

    #[inline]
    pub(crate) fn from_cp_orbit_twist_parity_coord(
        cube: &CornerCube333,
    ) -> CPOrbitTwistCoord {
        let orbit_corners = arrange_orbit_corners(cube.0);
        let relevant_corners = u8x16_swizzle(
            orbit_corners,
            i8x16( 0, 1, 2, 4, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1,-1),
        );

        let ud = u8x16_bitmask(relevant_corners);

        let ud_twist = ORBIT_STATE_LUT[ud as usize];

        CPOrbitTwistCoord(ud_twist)
    }

    #[inline]
    pub fn from_parity_coord(cube: &CornerCube333) -> ParityCoord {
        let values_12345 = u8x16_swizzle(
            cube.0,
            i8x16( 1, 2, 2, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 5,-1),
        );
        let values_67 = u8x16_swizzle(
            cube.0,
            i8x16( 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, -1, -1,-1),
        );

        let higher_left_12345 = v128_and(
            u8x16_lt(
                values_12345,
                u8x16_swizzle(
                    cube.0,
                    i8x16( 0, 0, 1, 0, 1, 2, 0, 1, 2, 3, 0, 1, 2, 3, 4,-1),
                ),
            ),
            u8x16_set1(1),
        );

        let higher_left_67 = v128_and(
            u8x16_lt(
                values_67,
                u8x16_swizzle(
                    cube.0,
                    i8x16( 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, -1, -1,-1),
                ),
            ),
            u8x16_set1(1),
        );

        let parity = v128_xor(higher_left_12345, higher_left_67);
        let parity = mm_sad_epu8(parity);
        let parity = u64x2_extract_lane::<0>(u32x4_shuffle::<0, 2, 0, 0>(parity, u64x2(0, 0)));
        let parity = (parity ^ (parity >> 32)) & 1;

        ParityCoord(parity == 1)
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