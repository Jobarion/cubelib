#[repr(align(16))]
#[derive(Debug, Clone, Copy)]
pub struct AlignedU64(pub [u64; 2]);

#[repr(align(16))]
#[derive(Debug, Clone, Copy)]
pub struct AlignedU8(pub [u8; 16]);

#[cfg(target_feature = "avx2")]
#[allow(dead_code)]
pub mod avx2 {
    use std::arch::x86_64::__m128i;

    //For loading const __m128i values
    pub union C {
        pub a: __m128i,
        pub a_u8: [u8; 16],
        pub a_u16: [u16; 8],
    }
}

#[cfg(target_feature = "neon")]
#[allow(dead_code)]
pub mod neon {
    use std::arch::aarch64::{uint8x16_t, uint8x8_t, vaddv_u8, vand_u8, vdup_n_u8, vreinterpret_s8_u8, vshl_u8};

    //For loading const uint8x16_t values
    pub union C16 {
        pub a: uint8x16_t,
        pub a_u8: [u8; 16],
        pub a_u16: [u16; 8],
    }

    //For loading const uint8x16_t values
    pub union C8 {
        pub a: uint8x8_t,
        pub a_u8: [u8; 8],
    }

    pub unsafe fn _mm_movemask_epi8_u8(data: uint8x8_t) -> u8 {
        let data = vand_u8(data, vdup_n_u8(1));
        let data = vshl_u8(data, vreinterpret_s8_u8(C8 { a_u8: [0, 1, 2, 3, 4, 5, 6, 7]}.a));
        vaddv_u8(data)
    }
}
