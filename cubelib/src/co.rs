use crate::puzzles::c333::{CornerCube333, Cube333};

pub trait COCountUD {
    fn co_count(&self) -> u8;
}

pub trait COCountFB {
    fn co_count(&self) -> u8;
}

pub trait COCountLR {
    fn co_count(&self) -> u8;
}

pub trait COCount {
    fn co_count_all(&self) -> (u8, u8, u8);
}

impl<CO: COCountUD + COCountLR + COCountFB>
COCount for CO
{
    fn co_count_all(&self) -> (u8, u8, u8) {
        (
            (COCountUD::co_count(self)),
            (COCountFB::co_count(self)),
            (COCountLR::co_count(self)),
        )
    }
}

impl COCountUD for Cube333 {

    fn co_count(&self) -> u8 {
        self.corners.co_count()
    }
}

impl COCountUD for CornerCube333 {

    #[cfg(target_feature = "avx2")]
    fn co_count(&self) -> u8 {
        unsafe {
            avx2::co_ud(self)
        }
    }

    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn co_count(&self) -> u8 {
        wasm32::co_ud(self)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_cmpgt_epi8, _mm_movemask_epi8, _mm_set1_epi8};
    use crate::puzzles::c333::CornerCube333;

    pub unsafe fn co_ud(cube: &CornerCube333) -> u8 {
        let co = _mm_and_si128(cube.0, _mm_set1_epi8(0x0F));
        let bad_corners = _mm_cmpgt_epi8(co, _mm_set1_epi8(0));
        let count = ((_mm_movemask_epi8(bad_corners) & 0xFF) as usize).count_ones();
        count as u8
    }
}

#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
mod wasm32 {
    use std::arch::wasm32::{u64x2, u8x16, v128, u8x16_gt, v128_and, u8x16_bitmask};
    use crate::cubie::CornerCubieCube;

    pub fn co_ud(cube: &CornerCubieCube) -> u8 {
        let co = v128_and(cube.0, u8x16set1(0x0F));
        let bad_corners = u8x16_gt(co, u64x2(0, 0));
        let count = ((u8x16_bitmask(bad_corners) & 0xFF) as usize).count_ones();
        count as u8
    }

    fn u8x16set1(a: u8) -> v128 {
        u8x16(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
    }
}