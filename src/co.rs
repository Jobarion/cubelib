use crate::cubie::{CornerCubieCube, CubieCube};

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

impl COCountUD for CubieCube {

    fn co_count(&self) -> u8 {
        self.corners.co_count()
    }
}

impl COCountUD for CornerCubieCube {

    #[cfg(target_feature = "avx2")]
    fn co_count(&self) -> u8 {
        unsafe {
            avx2::co_ud(self)
        }
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_movemask_epi8, _mm_set1_epi8};
    use crate::cubie::CornerCubieCube;

    pub unsafe fn co_ud(cube: &CornerCubieCube) -> u8 {
        let co = _mm_and_si128(cube.0, _mm_set1_epi8(0x0F));
        let bad_corners = _mm_cmpgt_epi8(co, _mm_set1_epi8(0));
        let count = ((_mm_movemask_epi8(bad_corners) & 0xFF) as usize).count_ones();
        count as u8
    }
}