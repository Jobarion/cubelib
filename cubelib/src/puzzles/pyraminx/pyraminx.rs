use crate::puzzles::puzzle::TurnableMut;
use crate::puzzles::pyraminx::PyraminxTurn;

//6 bytes for edges
    //4 bits empty, 3 bits for type, 1 for orientation.
    //UB UR UL RL RB LB
//2 bytes free
//4 bytes for centers
//4 bytes for tips
pub struct Pyraminx(
    #[cfg(target_feature = "avx2")]
    pub core::arch::x86_64::__m128i,
);

impl TurnableMut<PyraminxTurn> for Pyraminx {
    fn turn(&mut self, turn: PyraminxTurn) {

    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_add_epi8, _mm_and_si128, _mm_cmpeq_epi8, _mm_or_pd, _mm_or_si128, _mm_set1_epi8, _mm_set_epi64x, _mm_shuffle_epi8, _mm_srl_epi16, _mm_sub_epi8, _mm_xor_si128};

    use crate::alignment::avx2::C;
    use crate::puzzles::pyraminx::pyraminx::Pyraminx;
    use crate::puzzles::pyraminx::PyraminxTurn;

    const CENTER_TIP_ROT: [__m128i; 2] = unsafe { [
        C { a_u8: [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a,
        C { a_u8: [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a
    ] };

    const EO_FLIP: [__m128i; 4] = unsafe { [
        C { a_u8: [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //U
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //L
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //R
        C { a_u8: [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //B
    ] };

    const EDGE_SHUFFLE: [__m128i; 8] = unsafe { [
        C { a_u8: [2, 0, 1, 3, 4, 5, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //U
        C { a_u8: [1, 2, 0, 3, 4, 5, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //U'
        C { a_u8: [0, 1, 5, 2, 4, 3, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //L
        C { a_u8: [0, 1, 3, 5, 4, 2, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //L'
        C { a_u8: [0, 3, 2, 4, 1, 5, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //R
        C { a_u8: [0, 4, 2, 1, 3, 5, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //R'
        C { a_u8: [4, 1, 2, 3, 5, 0, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //B
        C { a_u8: [5, 1, 2, 3, 0, 4, 0xFF, 0xFF, 8, 9, 10, 11, 12, 13, 14, 15] }.a, //B'
    ] };

    unsafe fn turn(pyra: &mut Pyraminx, turn: PyraminxTurn) {
        let center_tips = _mm_shuffle_epi8(pyra.0, CENTER_TIP_ROT[turn.dir as usize]);
        let edges = _mm_xor_si128(pyra.0, EO_FLIP[turn.tip]);
        let edges = _mm_shuffle_epi8(edges, EDGE_SHUFFLE[turn.to_id()]);
        pyra.0 = _mm_or_si128(edges, _mm_and_si128(center_tips, _mm_set_epi64x(-1, 0)));
    }
}