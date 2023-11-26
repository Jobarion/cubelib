use crate::puzzles::puzzle::{InvertibleMut, TransformableMut, TurnableMut};
use crate::puzzles::pyraminx::{Direction, PyraminxTransformation, PyraminxTurn};

//6 bytes for edges
    //4 bits empty, 3 bits for type, 1 for orientation.
    //UB UR UL RL RB LB
//2 bytes free
//4 bytes for centers
//4 bytes for tips
#[derive(Copy, Clone, Debug)]
pub struct Pyraminx(
    #[cfg(target_feature = "avx2")]
    pub core::arch::x86_64::__m128i,
);

impl TurnableMut<PyraminxTurn> for Pyraminx {
    fn turn(&mut self, turn: PyraminxTurn) {
        #[cfg(target_feature = "avx2")]
        unsafe { avx2::turn(self, turn) }
    }
}

impl TransformableMut<PyraminxTransformation> for Pyraminx {
    fn transform(&mut self, _: PyraminxTransformation) {
        todo!()
    }
}

impl InvertibleMut for Pyraminx {
    fn invert(&mut self) {
        todo!()
    }
}

impl Default for Pyraminx {
    fn default() -> Self {
        #[cfg(target_feature = "avx2")]
        unsafe { avx2::new() }
    }
}

impl Pyraminx {
    pub fn get_tips(&self) -> [Option<Direction>; 4] {
        #[cfg(target_feature = "avx2")]
        let tips = unsafe { avx2::get_relative_tips(self) };
        tips.to_le_bytes()
            .map(|x| match x {
                0 => None,
                1 => Some(Direction::Clockwise),
                2 => Some(Direction::CounterClockwise),
                _ => unreachable!()
            })
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_extract_epi32, _mm_or_si128, _mm_set1_epi8, _mm_setr_epi32, _mm_setr_epi8, _mm_shuffle_epi8, _mm_xor_si128};

    use crate::alignment::avx2::C;
    use crate::puzzles::pyraminx::pyraminx::Pyraminx;
    use crate::puzzles::pyraminx::PyraminxTurn;

    const CENTER_TIP_ROT: [__m128i; 2] = unsafe { [
        C { a_u8: [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a,
        C { a_u8: [2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a
    ] };

    const EO_FLIP: [__m128i; 8] = unsafe { [
        C { a_u8: [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //U
        C { a_u8: [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //Ui
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //L
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //Li
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //R
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //Ri
        C { a_u8: [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //B
        C { a_u8: [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }.a, //Bi
    ] };

    const EDGE_SHUFFLE: [__m128i; 8] = unsafe { [
        C { a_u8: [2, 0, 1, 3, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //U
        C { a_u8: [1, 2, 0, 3, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //U'
        C { a_u8: [0, 1, 5, 2, 4, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //L
        C { a_u8: [0, 1, 3, 5, 4, 2, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //L'
        C { a_u8: [0, 3, 2, 4, 1, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //R
        C { a_u8: [0, 4, 2, 1, 3, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //R'
        C { a_u8: [4, 1, 2, 3, 5, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //B
        C { a_u8: [5, 1, 2, 3, 0, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 12, 13, 14, 15] }.a, //B'
    ] };

    const CENTER_MASK: [__m128i; 4] = unsafe { [
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0, 0, 0] }.a, //U
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0, 0] }.a, //L
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0] }.a, //R
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0] }.a, //B
    ] };

    const TIP_MASK: [__m128i; 4] = unsafe { [
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0] }.a, //U
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0, 0] }.a, //L
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0] }.a, //R
        C { a_u8: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF] }.a, //B
    ] };

    pub unsafe fn new() -> Pyraminx {
        Pyraminx(_mm_setr_epi8(0b0000, 0b0010, 0b0100, 0b0110, 0b1000, 0b1010, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0))
    }

    pub unsafe fn turn(pyra: &mut Pyraminx, turn: PyraminxTurn) {
        if turn.tip_only {
            let tip_mask = TIP_MASK[turn.tip as usize];
            let tips = _mm_and_si128(tip_mask, _mm_shuffle_epi8(CENTER_TIP_ROT[turn.dir as usize], pyra.0));
            let tips_unchanged = _mm_and_si128(pyra.0, _mm_xor_si128(tip_mask, _mm_set1_epi8(-1)));
            let tips = _mm_or_si128(tips, tips_unchanged);
            let pyra_unchanged = _mm_and_si128(pyra.0, _mm_setr_epi32(-1, -1, -1, 0));
            pyra.0 = _mm_or_si128(pyra_unchanged, tips);
        } else {
            let id = turn.to_id() >> 1;
            let center_mask = CENTER_MASK[turn.tip as usize];
            let center = _mm_and_si128(center_mask, _mm_shuffle_epi8(CENTER_TIP_ROT[turn.dir as usize], pyra.0));
            let center_unchanged = _mm_and_si128(pyra.0, _mm_xor_si128(center_mask, _mm_set1_epi8(-1)));
            let center = _mm_or_si128(center, center_unchanged);

            let edges = _mm_xor_si128(pyra.0, EO_FLIP[id]);
            let edges = _mm_shuffle_epi8(edges, EDGE_SHUFFLE[id]);
            pyra.0 = _mm_or_si128(edges, _mm_and_si128(center, _mm_setr_epi32(0, 0, -1, 0)));
        }

    }

    pub unsafe fn get_relative_tips(pyra: &Pyraminx) -> u32 {
        _mm_extract_epi32::<3>(pyra.0) as u32
    }
}