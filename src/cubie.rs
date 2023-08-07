use std::arch::x86_64::{__m128i, __m256i, _mm_add_epi8, _mm_and_si128, _mm_andnot_si128, _mm_or_si128, _mm_set1_epi8, _mm_set_epi64x, _mm_set_epi8, _mm_shuffle_epi8, _mm_slli_epi64, _mm_srli_epi16, _mm_sub_epi8, _mm_xor_si128};
use std::ops::Index;
use crate::cube::{Cube, Face, Turn};
use crate::cube::Face::*;

//http://kociemba.org/math/cubielevel.htm
pub struct CubieCube {
    //One byte per edge, 4 bits for id, 3 bits for eo (UD/FB/RL), 1 bit free
    //UB UR UF UL FR FL BR BL DF DR DB DL
    edges: __m128i,
    //One byte per corner, 3 bits for id, 2 bits free, 3 bits for co (from UD perspective)
    //UBL UBR UFR UFL DFL DFR DBR DBL
    corners: __m128i,
}

//For loading const __m128i values
union C {
    a: __m128i,
    b: [u8; 16],
}

impl CubieCube {

    pub fn new_solved() -> CubieCube {
        CubieCube {
            edges: unsafe { _mm_slli_epi64::<4>(_mm_set_epi8(0, 0, 0, 0, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0)) },
            corners: unsafe { _mm_slli_epi64::<5>(_mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 7, 6, 5, 4, 3, 2, 1, 0)) },
        }
    }
}

impl Cube for CubieCube {

    fn turn(&mut self, face: Face, turn_type: Turn) {
        unsafe {
            self.unsafe_turn(face, turn_type);
        }
    }
}

impl CubieCube {
    //UB UR UF UL FR FL BR BL DF DR DB DL
    // 0  1  2  3  4  5  6  7  8  9 10 11
    const TURN_EDGE_SHUFFLE: [[__m128i; 3]; 6] = unsafe {[
        [
            unsafe { C {b: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U
            unsafe { C {b: [2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U2
            unsafe { C {b: [1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //U'
        ],
        [
            unsafe { C {b: [0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D
            unsafe { C {b: [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D2
            unsafe { C {b: [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //D'
        ],
        [
            unsafe { C {b: [0, 1, 5, 3, 2, 8, 6, 7, 4, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F
            unsafe { C {b: [0, 1, 8, 3, 5, 4, 6, 7, 2, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F2
            unsafe { C {b: [0, 1, 4, 3, 8, 2, 6, 7, 5, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }  //F'
        ],
        [
            unsafe { C {b: [6, 1, 2, 3, 4, 5, 10, 0, 8, 9, 7, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B
            unsafe { C {b: [10, 1, 2, 3, 4, 5, 7, 6, 8, 9, 0, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B2
            unsafe { C {b: [7, 1, 2, 3, 4, 5, 0, 10, 8, 9, 6, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B'
        ],
        [
            unsafe { C {b: [0, 1, 2, 7, 4, 3, 6, 11, 8, 9, 10, 5, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L
            unsafe { C {b: [0, 1, 2, 11, 4, 7, 6, 5, 8, 9, 10, 3, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L2
            unsafe { C {b: [0, 1, 2, 5, 4, 11, 6, 3, 8, 9, 10, 7, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L'
        ],
        [
            unsafe { C {b: [0, 4, 2, 3, 9, 5, 1, 7, 8, 6, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R
            unsafe { C {b: [0, 9, 2, 3, 6, 5, 4, 7, 8, 1, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R2
            unsafe { C {b: [0, 6, 2, 3, 1, 5, 9, 7, 8, 4, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R'
        ]
    ]};

    //UBL UBR UFR UFL DFL DFR DBR DBL
    //  0   1   2   3   4   5   6   7
    const TURN_CORNER_SHUFFLE: [[__m128i; 3]; 6] = unsafe {[
        [
            unsafe { C {b: [3, 0, 1, 2, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U
            unsafe { C {b: [2, 3, 0, 1, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U2
            unsafe { C {b: [1, 2, 3, 0, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //U'
        ],
        [
            unsafe { C {b: [0, 1, 2, 3, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D
            unsafe { C {b: [0, 1, 2, 3, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D2
            unsafe { C {b: [0, 1, 2, 3, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //D'
        ],
        [
            unsafe { C {b: [0, 1, 3, 4, 5, 2, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F
            unsafe { C {b: [0, 1, 4, 5, 2, 3, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F2
            unsafe { C {b: [0, 1, 5, 2, 3, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //F'
        ],
        [
            unsafe { C {b: [1, 6, 2, 3, 4, 5, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B
            unsafe { C {b: [6, 7, 2, 3, 4, 5, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B2
            unsafe { C {b: [7, 0, 2, 3, 4, 5, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //B'
        ],
        [
            unsafe { C {b: [7, 1, 2, 0, 3, 5, 6, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L
            unsafe { C {b: [4, 1, 2, 7, 0, 5, 6, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L2
            unsafe { C {b: [3, 1, 2, 4, 7, 5, 6, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //L'
        ],
        [
            unsafe { C {b: [0, 2, 5, 3, 4, 6, 1, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R
            unsafe { C {b: [0, 5, 6, 3, 4, 1, 2, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R2
            unsafe { C {b: [0, 6, 1, 3, 4, 2, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF] }.a }, //R'
        ],
    ]};

    const TURN_EO_FLIP: [__m128i; 6] = unsafe{[
        unsafe { C {b: [0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//U
        unsafe { C {b: [0, 0, 0, 0, 0, 0, 0, 0, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0] }.a },//D
        unsafe { C {b: [0, 0, 0b00000100, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0, 0, 0, 0] }.a },//F
        unsafe { C {b: [0b00000100, 0, 0, 0, 0, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0, 0] }.a },//B
        unsafe { C {b: [0, 0, 0, 0b00000010, 0, 0b00000010, 0, 0b00000010, 0, 0, 0, 0b00000010, 0, 0, 0, 0] }.a },//L
        unsafe { C {b: [0, 0b00000010, 0, 0, 0b00000010, 0, 0b00000010, 0, 0, 0b00000010, 0, 0, 0, 0, 0, 0] }.a },//R
    ]};

    const CO_OVERFLOW_MASK: __m128i = unsafe{ C {b: [0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0, 0, 0, 0, 0, 0, 0, 0] }.a };
    const TURN_CO_CHANGE: [__m128i; 6] = unsafe{[
        unsafe { C {b: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//U
        unsafe { C {b: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//D
        unsafe { C {b: [1, 1, 2, 3, 2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//F
        unsafe { C {b: [2, 3, 1, 1, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//B
        unsafe { C {b: [3, 1, 1, 2, 3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//L
        unsafe { C {b: [1, 2, 3, 1, 1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0] }.a },//R
    ]};


    #[target_feature(enable = "avx2")]
    unsafe fn unsafe_turn(&mut self, face: Face, turn_type: Turn) {
        self.corners = _mm_shuffle_epi8(self.corners, CubieCube::TURN_CORNER_SHUFFLE[face as usize][turn_type as usize]);
        self.edges = _mm_shuffle_epi8(self.edges, CubieCube::TURN_EDGE_SHUFFLE[face as usize][turn_type as usize]);
        if turn_type != Turn::Half {
            self.edges = _mm_xor_si128(self.edges, CubieCube::TURN_EO_FLIP[face as usize]);
            //Valid COs are 00, 01, 10. When we move, we don't add 0, 1, 2 (no change, clockwise, counter-clockwise), but we add 1, 2, 3 to force overflowing into the next bit.
            //This code either subtracts 1 if there is no overflow (because we added 1 too much before), or 4, because this gives us the original addition mod 3.
            let corners_tmp = _mm_add_epi8(self.corners, CubieCube::TURN_CO_CHANGE[face as usize]);
            let overflow_bits = _mm_and_si128(corners_tmp, CubieCube::CO_OVERFLOW_MASK);
            let not_overflow = _mm_srli_epi16::<2>(_mm_andnot_si128(corners_tmp, CubieCube::CO_OVERFLOW_MASK));
            let overflow_sub = _mm_or_si128(overflow_bits, not_overflow);
            self.corners = _mm_sub_epi8(corners_tmp, overflow_sub);
        }
    }
}
#[cfg(test)]
mod cubie_tests {
    use std::arch::x86_64::{__m128i, _mm_store_si128};
    use crate::cube::{Cube, Face, Turn};

    #[test]
    fn test_u() {
        test_face(Face::Up);
    }

    #[test]
    fn test_f() {
        test_face(Face::Front);
    }

    #[test]
    fn test_r() {
        test_face(Face::Right);
    }

    #[test]
    fn test_d() {
        test_face(Face::Down);
    }

    #[test]
    fn test_b() {
        test_face(Face::Back);
    }

    #[test]
    fn test_l() {
        test_face(Face::Left);
    }

    //Tests [R2;U2]x3 and [R;U]x6 style algorithms
    #[test]
    fn test_simple_algs() {
        for a in 0..6 {
            for b in 0..6 {
                if a == b {
                    continue;
                }
                test_ht_faces(Face::from(a), Face::from(b));
                test_qt_faces(Face::from(a), Face::from(b));
            }
        }
    }

    #[test]
    fn test_t_perm() {
        let mut cube = super::CubieCube::new_solved();
        let old_corners = cube.corners;
        let old_edges = cube.edges;

        for _ in 0..4 {
            //T perm
            cube.turn(Face::Right, Turn::Clockwise);
            cube.turn(Face::Up, Turn::Clockwise);
            cube.turn(Face::Right, Turn::CounterClockwise);
            cube.turn(Face::Up, Turn::CounterClockwise);
            cube.turn(Face::Right, Turn::CounterClockwise);
            cube.turn(Face::Front, Turn::Clockwise);
            cube.turn(Face::Right, Turn::Half);
            cube.turn(Face::Up, Turn::CounterClockwise);
            cube.turn(Face::Right, Turn::CounterClockwise);
            cube.turn(Face::Up, Turn::CounterClockwise);
            cube.turn(Face::Right, Turn::Clockwise);
            cube.turn(Face::Up, Turn::Clockwise);
            cube.turn(Face::Right, Turn::CounterClockwise);
            cube.turn(Face::Front, Turn::CounterClockwise);

            cube.turn(Face::Up, Turn::Half);
        }

        unsafe {
            assert_eq_m128(old_edges, cube.edges, "Edges not equal");
            assert_eq_m128(old_corners, cube.corners, "Corners not equal");
        }
    }

    fn test_qt_faces(a: Face, b: Face) {
        let mut cube = super::CubieCube::new_solved();
        let old_corners = cube.corners;
        let old_edges = cube.edges;

        for _ in 0..6 {
            cube.turn(a, Turn::Clockwise);
            cube.turn(b, Turn::Clockwise);
            cube.turn(a, Turn::CounterClockwise);
            cube.turn(b, Turn::CounterClockwise);
        }

        unsafe {
            assert_eq_m128(old_edges, cube.edges, "Edges not equal");
            assert_eq_m128(old_corners, cube.corners, "Corners not equal");
        }
    }

    fn test_ht_faces(a: Face, b: Face) {
        let mut cube = super::CubieCube::new_solved();
        let old_corners = cube.corners;
        let old_edges = cube.edges;
        for _ in 0..6 {
            cube.turn(a, Turn::Half);
            cube.turn(b, Turn::Half);
        }

        unsafe {
            assert_eq_m128(old_edges, cube.edges, "Edges not equal");
            assert_eq_m128(old_corners, cube.corners, "Corners not equal");
        }
    }

    fn test_face(face: Face) {
        let mut cube = super::CubieCube::new_solved();
        let old_corners = cube.corners;
        let old_edges = cube.edges;
        cube.turn(face, Turn::Clockwise);
        cube.turn(face, Turn::Half);
        cube.turn(face, Turn::CounterClockwise);
        cube.turn(face, Turn::Half);

        unsafe {
            assert_eq_m128(old_edges, cube.edges, "Edges not equal");
            assert_eq_m128(old_corners, cube.corners, "Corners not equal");
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn assert_eq_m128(a: __m128i, b: __m128i, msg: &str) {
        let mut a_arr = [0u64; 2];
        _mm_store_si128(a_arr.as_mut_ptr() as *mut __m128i, a);
        let mut b_arr = [0u64; 2];
        _mm_store_si128(b_arr.as_mut_ptr() as *mut __m128i, b);
        if a_arr != b_arr {
            panic!("{}", msg);
        }
    }
}