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
