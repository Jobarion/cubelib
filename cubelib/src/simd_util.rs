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
    use std::arch::aarch64::{int8x16_t, uint16x8_t, uint8x16_t, uint8x8_t};

    //For loading const uint8x16_t values
    pub union C16 {
        pub a: uint8x16_t,
        pub a_16: uint16x8_t,
        pub a_i: int8x16_t,
        pub a_u8: [u8; 16],
        pub a_i8: [i8; 16],
        pub a_u16: [u16; 8],
        pub a_i32: [i32; 4],
    }

    //For loading const uint8x16_t values
    pub union C8 {
        pub a: uint8x8_t,
        pub a_u8: [u8; 8],
        pub a_i8: [i8; 8],
    }
}
