use std::arch::x86_64::{__m128i, __m256i, _mm_add_epi8, _mm_and_si128, _mm_andnot_si128, _mm_extract_epi64, _mm_load_si128, _mm_loadl_epi64, _mm_or_si128, _mm_set1_epi8, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi8, _mm_slli_epi32, _mm_slli_epi64, _mm_srli_epi16, _mm_srli_epi32, _mm_store_si128, _mm_sub_epi8, _mm_xor_si128};
use std::fmt::{Display, Formatter};
use crate::alignment::{AlignedU64, AlignedU8, C};
use crate::coord::EOCoordAll;
use crate::cube::{Color, Corner, CornerPosition, Cube, Edge, EdgePosition, Face, Invertible, Move, NewSolved, Turn, Turnable};
use crate::cube::Color::*;
use crate::cube::EdgePosition::*;
use crate::cube::CornerPosition::*;
use crate::cube::Face::*;
use crate::eo::EOCount;

//One byte per edge, 4 bits for id, 3 bits for eo (UD/FB/RL), 1 bit free
//UB UR UF UL FR FL BR BL DF DR DB DL
#[derive(Clone, Copy)]
pub struct EdgeCubieCube(pub __m128i);

impl EdgeCubieCube {

    pub fn get_edges(&self) -> [Edge; 12] {
        unsafe {
            self.unsafe_get_edges()
        }
    }

    pub fn get_edges_raw(&self) -> [u64; 2] {
        unsafe {
            self.unsafe_get_edges_raw()
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_get_edges_raw(&self) -> [u64; 2] {
        let mut a_arr = AlignedU64([0u64; 2]).0;
        _mm_store_si128(a_arr.as_mut_ptr() as *mut __m128i, self.0);
        a_arr[1] &= CubieCube::VALID_EDGE_MASK_HI;
        a_arr
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_new_solved() -> EdgeCubieCube {
        EdgeCubieCube(unsafe { _mm_slli_epi64::<4>(_mm_set_epi8(0, 0, 0, 0, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)) })
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_get_edges(&self) -> [Edge; 12] {
        let mut edges = unsafe {
            let mut a_arr = AlignedU64([0u64; 2]).0;
            _mm_store_si128(a_arr.as_mut_ptr() as *mut __m128i, self.0);
            a_arr
        };
        let mut edge_arr = [Edge {id: 0, oriented_ud: true, oriented_fb: true, oriented_rl: true}; 12];

        for eid in 0..12 {
            let arr_id = eid / 8;
            let edge = (edges[arr_id] & 0xFF) as u8;
            edges[arr_id] >>= 8;

            let rl = edge & 0b0010 == 0;
            let fb = edge & 0b0100 == 0;
            let ud = edge & 0b1000 == 0;

            edge_arr[eid] = Edge {id: edge >> 4 , oriented_ud: ud, oriented_fb: fb, oriented_rl: rl};
        }

        edge_arr
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_turn(&mut self, face: Face, turn_type: Turn) {
        self.0 = _mm_shuffle_epi8(self.0, CubieCube::TURN_EDGE_SHUFFLE[face as usize][turn_type as usize]);
        if turn_type != Turn::Half {
            self.0 = _mm_xor_si128(self.0, CubieCube::TURN_EO_FLIP[face as usize]);
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_invert(&mut self) {
        let edge_ids = unsafe {
            let mut a_arr = AlignedU8([0u8; 16]);
            _mm_store_si128(a_arr.0.as_mut_ptr() as *mut __m128i, _mm_srli_epi32::<4>(_mm_and_si128(self.0, _mm_set1_epi8(0xF0_u8 as i8))));
            a_arr
        };
        //This essentially calculates the inverse of _mm_shuffle_epi8(solved_cube.edges, self.edges), same for corners
        let mut edge_shuffle = AlignedU8([0u8; 16]);
        let edges = edge_ids.0;
        for i in 0..12 {
            edge_shuffle.0[edges[i] as usize] = i as u8;
        }
        let edge_shuffle_mask = _mm_load_si128(edge_shuffle.0.as_ptr() as *const __m128i);

        //Splice together the edge permutation, and the EO of the edges on the inverse (see niss prediction to see how this works)
        let ep = _mm_and_si128(_mm_shuffle_epi8(_mm_shuffle_epi8(self.0, edge_shuffle_mask), edge_shuffle_mask), _mm_set1_epi8(0xF0_u8 as i8));
        let eo_shuffle = _mm_shuffle_epi8(self.0, _mm_srli_epi32::<4>(ep));
        let eo = _mm_and_si128(eo_shuffle, _mm_set1_epi8(0b1110));

        self.0 = _mm_or_si128(ep, eo);
    }
}

impl Turnable for EdgeCubieCube {

    #[inline]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        unsafe {
            self.unsafe_turn(face, turn);
        }
    }
}

impl Invertible for EdgeCubieCube {

    #[inline]
    fn invert(&mut self) {
        unsafe {
            self.unsafe_invert();
        }
    }
}

impl NewSolved for EdgeCubieCube {

    #[inline]
    fn new_solved() -> Self {
        unsafe {
            Self::unsafe_new_solved()
        }
    }
}

impl Turnable for CornerCubieCube {

    #[inline]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        unsafe {
            self.unsafe_turn(face, turn);
        }
    }
}

impl Invertible for CornerCubieCube {

    #[inline]
    fn invert(&mut self) {
        unsafe {
            self.unsafe_invert();
        }
    }
}

impl NewSolved for CornerCubieCube {

    #[inline]
    fn new_solved() -> Self {
        unsafe {
            Self::unsafe_new_solved()
        }
    }
}

//One byte per corner, 3 bits for id, 2 bits free, 3 bits for co (from UD perspective)
//UBL UBR UFR UFL DFL DFR DBR DBL
#[derive(Clone, Copy)]
pub struct CornerCubieCube(pub __m128i);

impl CornerCubieCube {

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsafe_new_solved() -> CornerCubieCube {
        CornerCubieCube(unsafe { _mm_slli_epi64::<5>(_mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 7, 6, 5, 4, 3, 2, 1, 0)) })
    }

    fn get_corners(&self) -> [Corner; 8] {
        unsafe {
            self.unsafe_get_corners()
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsafe_get_corners(&self) -> [Corner; 8] {
        let mut corner_bits = _mm_extract_epi64::<0>(self.0) as u64;
        let mut corner_arr = [Corner {id: 0, orientation: 0}; 8];
        for cid in 0..8 {
            let corner = (corner_bits & 0xFF) as u8;
            corner_bits >>= 8;
            corner_arr[cid] = Corner {id: corner >> 5 , orientation: corner & 0x7};
        }
        corner_arr
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsafe_turn(&mut self, face: Face, turn_type: Turn) {
        self.0 = _mm_shuffle_epi8(self.0, CubieCube::TURN_CORNER_SHUFFLE[face as usize][turn_type as usize]);
        if turn_type != Turn::Half {
            //Valid COs are 00, 01, 10. When we move, we don't add 0, 1, 2 (no change, clockwise, counter-clockwise), but we add 1, 2, 3 to force overflowing into the next bit.
            //This code either subtracts 1 if there is no overflow (because we added 1 too much before), or 4, because this gives us the original addition mod 3.
            let corners_tmp = _mm_add_epi8(self.0, CubieCube::TURN_CO_CHANGE[face as usize]);
            let overflow_bits = _mm_and_si128(corners_tmp, CubieCube::CO_OVERFLOW_MASK);
            let not_overflow = _mm_srli_epi16::<2>(_mm_andnot_si128(corners_tmp, CubieCube::CO_OVERFLOW_MASK));
            let overflow_sub = _mm_or_si128(overflow_bits, not_overflow);
            self.0 = _mm_sub_epi8(corners_tmp, overflow_sub);
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn unsafe_invert(&mut self) {
        let mut corner_ids = unsafe {
            (_mm_extract_epi64::<0>(_mm_srli_epi32::<5>(_mm_and_si128(self.0, _mm_set1_epi8(0xE0_u8 as i8)))) as u64).to_le_bytes()
        };

        let mut corner_shuffle = corner_ids.clone();
        for i in 0..8 {
            corner_shuffle[corner_ids[i] as usize] = i as u8;
        }
        let corner_shuffle_mask = _mm_loadl_epi64(corner_shuffle.as_ptr() as *const __m128i);

        //Splice together the corner permutation, and the CO of the corners on the inverse (see niss prediction to see how this works)
        //Also switch CO 1 <-> 2,  CO 0 stays the same
        let cp = _mm_and_si128(_mm_shuffle_epi8(_mm_shuffle_epi8(self.0, corner_shuffle_mask), corner_shuffle_mask), _mm_set1_epi8(0b11100000_u8 as i8));
        let co_shuffle = _mm_shuffle_epi8(self.0, _mm_srli_epi32::<5>(cp));
        let tmp = _mm_and_si128(_mm_add_epi8(co_shuffle, _mm_set1_epi8(1)), _mm_set1_epi8(2));
        let co_flip_mask = _mm_or_si128(tmp, _mm_srli_epi32::<1>(tmp));
        let co = _mm_and_si128(_mm_xor_si128(co_shuffle, co_flip_mask), _mm_set1_epi8(7));

        self.0 = _mm_or_si128(cp, co);
    }
}

//http://kociemba.org/math/cubielevel.htm
#[derive(Clone, Copy)]
pub struct CubieCube {
    pub(crate) edges: EdgeCubieCube,
    pub(crate) corners: CornerCubieCube,
}

impl CubieCube {

    pub fn get_corners_raw(&self) -> u64 {
        unsafe { self.unsafe_get_corners_raw() }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_get_corners_raw(&self) -> u64 {
        _mm_extract_epi64::<0>(self.corners.0) as u64
    }

    pub fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

impl Display for CubieCube {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_display(f)
    }
}

impl Turnable for CubieCube {

    #[inline]
    fn turn(&mut self, m: Move) {
        let Move(face, turn) = m;
        unsafe {
            self.unsafe_turn(face, turn);
        }
    }
}

impl Invertible for CubieCube {

    #[inline]
    fn invert(&mut self) {
        unsafe {
            self.unsafe_invert();
        }
    }
}

impl NewSolved for CubieCube {

    #[inline]
    fn new_solved() -> CubieCube {
        CubieCube {
            edges: EdgeCubieCube::new_solved(),
            corners: CornerCubieCube::new_solved(),
        }
    }
}

impl Cube for CubieCube {

    fn get_facelets(&self) -> [[Color; 9]; 6] {

        let corners = self.corners.get_corners();
        let edges = self.edges.get_edges();
        let mut facelets = [[None; 9]; 6];

        //There has to be a better way
        let c = |id: CornerPosition, twist: u8| {
            let corner = corners[id as usize];
            let twist_id = (3 - corner.orientation + twist) % 3;
            CubieCube::CORNER_COLORS[corner.id as usize][twist_id as usize]
        };

        let e = |id: EdgePosition, flip: bool| {
            let edge = edges[id as usize];
            let eo_id = !(edge.oriented_fb ^ flip) as usize;
            CubieCube::EDGE_COLORS[edge.id as usize][eo_id as usize]
        };

        facelets[Up][0] = c(UBL, 0);
        facelets[Up][1] = e(UB, false);
        facelets[Up][2] = c(UBR, 0);
        facelets[Up][3] = e(UL, false);
        facelets[Up][4] = White;
        facelets[Up][5] = e(UR, false);
        facelets[Up][6] = c(UFL, 0);
        facelets[Up][7] = e(UF, false);
        facelets[Up][8] = c(UFR, 0);

        facelets[Down][0] = c(DFL, 0);
        facelets[Down][1] = e(DF, false);
        facelets[Down][2] = c(DFR, 0);
        facelets[Down][3] = e(DL, false);
        facelets[Down][4] = Yellow;
        facelets[Down][5] = e(DR, false);
        facelets[Down][6] = c(DBL, 0);
        facelets[Down][7] = e(DB, false);
        facelets[Down][8] = c(DBR, 0);

        facelets[Front][0] = c(UFL, 1);
        facelets[Front][1] = e(UF, true);
        facelets[Front][2] = c(UFR, 2);
        facelets[Front][3] = e(FL, false);
        facelets[Front][4] = Green;
        facelets[Front][5] = e(FR, false);
        facelets[Front][6] = c(DFL, 2);
        facelets[Front][7] = e(DF, true);
        facelets[Front][8] = c(DFR, 1);

        facelets[Back][0] = c(UBR, 1);
        facelets[Back][1] = e(UB, true);
        facelets[Back][2] = c(UBL, 2);
        facelets[Back][3] = e(BR, false);
        facelets[Back][4] = Blue;
        facelets[Back][5] = e(BL, false);
        facelets[Back][6] = c(DBR, 2);
        facelets[Back][7] = e(DB, true);
        facelets[Back][8] = c(DBL, 1);

        facelets[Left][0] = c(UBL, 1);
        facelets[Left][1] = e(UL, true);
        facelets[Left][2] = c(UFL, 2);
        facelets[Left][3] = e(BL, true);
        facelets[Left][4] = Orange;
        facelets[Left][5] = e(FL, true);
        facelets[Left][6] = c(DBL, 2);
        facelets[Left][7] = e(DL, true);
        facelets[Left][8] = c(DFL, 1);

        facelets[Right][0] = c(UFR, 1);
        facelets[Right][1] = e(UR, true);
        facelets[Right][2] = c(UBR, 2);
        facelets[Right][3] = e(FR, true);
        facelets[Right][4] = Red;
        facelets[Right][5] = e(BR, true);
        facelets[Right][6] = c(DFR, 2);
        facelets[Right][7] = e(DR, true);
        facelets[Right][8] = c(DBR, 1);

        facelets
    }
}

impl CubieCube {

    pub(crate) const BAD_EDGE_MASK_UD: u64 = 0x0808080808080808;
    pub(crate) const BAD_EDGE_MASK_FB: u64 = 0x0404040404040404;
    pub(crate) const BAD_EDGE_MASK_RL: u64 = 0x0202020202020202;
    pub(crate) const VALID_EDGE_MASK_HI: u64 = 0x00000000FFFFFFFF;

    pub const CORNER_COLORS: [[Color; 3]; 8] = [
        [White, Orange, Blue],
        [White, Blue, Red],
        [White, Red, Green],
        [White, Green, Orange],
        [Yellow, Orange, Green],
        [Yellow, Green, Red],
        [Yellow, Red, Blue],
        [Yellow, Blue, Orange]
    ];

    const EDGE_COLORS: [[Color; 2]; 12] = [
        [White, Blue],
        [White, Red],
        [White, Green],
        [White, Orange],
        [Green, Red],
        [Green, Orange],
        [Blue, Red],
        [Blue, Orange],
        [Yellow, Green],
        [Yellow, Red],
        [Yellow, Blue],
        [Yellow, Orange]
    ];

    //UB UR UF UL FR FL BR BL DF DR DB DL
    // 0  1  2  3  4  5  6  7  8  9 10 11
    const TURN_EDGE_SHUFFLE: [[__m128i; 3]; 6] = unsafe {[
        [
            unsafe { C { a_u8: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U
            unsafe { C { a_u8: [2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U2
            unsafe { C { a_u8: [1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //U'
        ],
        [
            unsafe { C { a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D
            unsafe { C { a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D2
            unsafe { C { a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //D'
        ],
        [
            unsafe { C { a_u8: [0, 1, 5, 3, 2, 8, 6, 7, 4, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F
            unsafe { C { a_u8: [0, 1, 8, 3, 5, 4, 6, 7, 2, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F2
            unsafe { C { a_u8: [0, 1, 4, 3, 8, 2, 6, 7, 5, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //F'
        ],
        [
            unsafe { C { a_u8: [6, 1, 2, 3, 4, 5, 10, 0, 8, 9, 7, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B
            unsafe { C { a_u8: [10, 1, 2, 3, 4, 5, 7, 6, 8, 9, 0, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B2
            unsafe { C { a_u8: [7, 1, 2, 3, 4, 5, 0, 10, 8, 9, 6, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B'
        ],
        [
            unsafe { C { a_u8: [0, 1, 2, 7, 4, 3, 6, 11, 8, 9, 10, 5, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L
            unsafe { C { a_u8: [0, 1, 2, 11, 4, 7, 6, 5, 8, 9, 10, 3, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L2
            unsafe { C { a_u8: [0, 1, 2, 5, 4, 11, 6, 3, 8, 9, 10, 7, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L'
        ],
        [
            unsafe { C { a_u8: [0, 4, 2, 3, 9, 5, 1, 7, 8, 6, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R
            unsafe { C { a_u8: [0, 9, 2, 3, 6, 5, 4, 7, 8, 1, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R2
            unsafe { C { a_u8: [0, 6, 2, 3, 1, 5, 9, 7, 8, 4, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R'
        ]
    ]};

    //UBL UBR UFR UFL DFL DFR DBR DBL
    //  0   1   2   3   4   5   6   7
    const TURN_CORNER_SHUFFLE: [[__m128i; 3]; 6] = unsafe {[
        [
            unsafe { C { a_u8: [3, 0, 1, 2, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U
            unsafe { C { a_u8: [2, 3, 0, 1, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U2
            unsafe { C { a_u8: [1, 2, 3, 0, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U'
        ],
        [
            unsafe { C { a_u8: [0, 1, 2, 3, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D
            unsafe { C { a_u8: [0, 1, 2, 3, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D2
            unsafe { C { a_u8: [0, 1, 2, 3, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D'
        ],
        [
            unsafe { C { a_u8: [0, 1, 3, 4, 5, 2, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F
            unsafe { C { a_u8: [0, 1, 4, 5, 2, 3, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F2
            unsafe { C { a_u8: [0, 1, 5, 2, 3, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F'
        ],
        [
            unsafe { C { a_u8: [1, 6, 2, 3, 4, 5, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B
            unsafe { C { a_u8: [6, 7, 2, 3, 4, 5, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B2
            unsafe { C { a_u8: [7, 0, 2, 3, 4, 5, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B'
        ],
        [
            unsafe { C { a_u8: [7, 1, 2, 0, 3, 5, 6, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L
            unsafe { C { a_u8: [4, 1, 2, 7, 0, 5, 6, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L2
            unsafe { C { a_u8: [3, 1, 2, 4, 7, 5, 6, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L'
        ],
        [
            unsafe { C { a_u8: [0, 2, 5, 3, 4, 6, 1, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R
            unsafe { C { a_u8: [0, 5, 6, 3, 4, 1, 2, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R2
            unsafe { C { a_u8: [0, 6, 1, 3, 4, 2, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R'
        ],
    ]};

    //UB UR UF UL FR FL BR BL DF DR DB DL
    // 0  1  2  3  4  5  6  7  8  9 10 11
    const TURN_EO_FLIP: [__m128i; 6] = unsafe{[
        unsafe { C { a_u8: [0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//U
        unsafe { C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0] }.a },//D
        unsafe { C { a_u8: [0, 0, 0b00000100, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0, 0, 0, 0] }.a },//F
        unsafe { C { a_u8: [0b00000100, 0, 0, 0, 0, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0, 0] }.a },//B
        unsafe { C { a_u8: [0, 0, 0, 0b00000010, 0, 0b00000010, 0, 0b00000010, 0, 0, 0, 0b00000010, 0, 0, 0, 0] }.a },//L
        unsafe { C { a_u8: [0, 0b00000010, 0, 0, 0b00000010, 0, 0b00000010, 0, 0, 0b00000010, 0, 0, 0, 0, 0, 0] }.a },//R
    ]};

    const CO_OVERFLOW_MASK: __m128i = unsafe{ C { a_u8: [0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0, 0, 0, 0, 0, 0, 0, 0] }.a };

    const TURN_CO_CHANGE: [__m128i; 6] = unsafe{[
        unsafe { C { a_u8: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//U
        unsafe { C { a_u8: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//D
        unsafe { C { a_u8: [1, 1, 2, 3, 2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//F
        unsafe { C { a_u8: [2, 3, 1, 1, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//B
        unsafe { C { a_u8: [3, 1, 1, 2, 3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//L
        unsafe { C { a_u8: [1, 2, 3, 1, 1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//R
    ]};

    #[inline]
    unsafe fn unsafe_turn(&mut self, face: Face, turn_type: Turn) {
        self.edges.unsafe_turn(face, turn_type);
        self.corners.unsafe_turn(face, turn_type);
    }

    #[inline]
    unsafe fn unsafe_invert(&mut self) {
        self.edges.unsafe_invert();
        self.corners.unsafe_invert();
    }
}