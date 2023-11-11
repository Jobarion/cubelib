use crate::steps::coord::Coord;
use crate::puzzles::c333::{Cube333, EdgeCube333};

//Edge orientation on the respective axis
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordAll(pub EOCoordUD, pub EOCoordFB, pub EOCoordLR);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordUD(pub(crate) u16);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordFB(pub(crate) u16);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordLR(pub(crate) u16);

impl Coord<2048> for EOCoordUD {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EOCoordUD {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2048> for EOCoordFB {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EOCoordFB {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2048> for EOCoordLR {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EOCoordLR {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<&EdgeCube333> for EOCoordAll {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_eocoord_all(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCubieCube) -> Self {
        wasm32::from_eocoord_all(value)
    }
}

impl From<&EdgeCube333> for EOCoordUD {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_eocoord_ud(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCubieCube) -> Self {
        wasm32::from_eocoord_ud(value)
    }
}

impl From<&Cube333> for EOCoordUD {
    fn from(value: &Cube333) -> Self {
        EOCoordUD::from(&value.edges)
    }
}

impl From<&EdgeCube333> for EOCoordFB {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_eocoord_fb(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCubieCube) -> Self {
        wasm32::from_eocoord_fb(value)
    }
}

impl From<&Cube333> for EOCoordFB {
    fn from(value: &Cube333) -> Self {
        EOCoordFB::from(&value.edges)
    }
}

impl From<&EdgeCube333> for EOCoordLR {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCube333) -> Self {
        unsafe { avx2::unsafe_from_eocoord_lr(value) }
    }

    #[inline]
    #[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
    fn from(value: &EdgeCubieCube) -> Self {
        wasm32::from_eocoord_lr(value)
    }
}

impl From<&Cube333> for EOCoordLR {
    fn from(value: &Cube333) -> Self {
        EOCoordLR::from(&value.edges)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_movemask_epi8, _mm_setr_epi8, _mm_slli_epi64};

    use crate::puzzles::c333::steps::eo::coords::{EOCoordAll, EOCoordFB, EOCoordLR, EOCoordUD};
    use crate::puzzles::c333::EdgeCube333;

    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_all(value: &EdgeCube333) -> EOCoordAll {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_setr_epi8( 0x0F,
                0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00,
                0x00),
        );
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        let lr = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordAll(EOCoordUD(ud), EOCoordFB(fb), EOCoordLR(lr))
    }

    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_ud(value: &EdgeCube333) -> EOCoordUD {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_setr_epi8( 0x0F,
                0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00,
                0x00),
        );
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        EOCoordUD(ud)
    }

    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_fb(value: &EdgeCube333) -> EOCoordFB {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_setr_epi8( 0x0F,
                0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00,
                0x00),
        );
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        EOCoordFB(fb)
    }

    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_lr(value: &EdgeCube333) -> EOCoordLR {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_setr_epi8( 0x0F,
                0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00,
                0x00),
        );
        let rl = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordLR(rl)
    }
}

#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
mod wasm32 {
    use std::arch::wasm32::{u32x4_shl, u8x16, v128_and, u8x16_bitmask};
    use crate::puzzles::c333::EdgeCube333;
    use crate::puzzles::c333::steps::eo::coords::{EOCoordAll, EOCoordFB, EOCoordLR, EOCoordUD};

    #[inline]
    pub(crate) fn from_eocoord_ud(value: &EdgeCube333) -> EOCoordUD {
        EOCoordUD(get_eocoord::<4>(value))
    }

    #[inline]
    pub(crate) fn from_eocoord_fb(value: &EdgeCube333) -> EOCoordFB {
        EOCoordFB(get_eocoord::<5>(value))
    }

    #[inline]
    pub(crate) fn from_eocoord_lr(value: &EdgeCube333) -> EOCoordLR {
        EOCoordLR(get_eocoord::<6>(value))
    }

    #[inline]
    fn get_eocoord<const SHL: u32>(value: &EdgeCube333) -> u16 {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = v128_and(
            value.0,
            u8x16(0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                  0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00),
        );
        u8x16_bitmask(u32x4_shl(no_db_edge, SHL))
    }

    #[inline]
    pub(crate) fn from_eocoord_all(value: &EdgeCube333) -> EOCoordAll {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = v128_and(
            value.0,
            u8x16(0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                  0x0F, 0x0F, 0x00, 0x00, 0x00, 0x00, 0x00),
        );
        let ud = u8x16_bitmask(u32x4_shl(no_db_edge, 4));
        let fb = u8x16_bitmask(u32x4_shl(no_db_edge, 5));
        let lr = u8x16_bitmask(u32x4_shl(no_db_edge, 6));
        EOCoordAll(EOCoordUD(ud), EOCoordFB(fb), EOCoordLR(lr))
    }
}