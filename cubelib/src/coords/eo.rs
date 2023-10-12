use crate::coords::coord::Coord;
use crate::cube::Edge;
use crate::cubie::{CubieCube, EdgeCubieCube};

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

impl From<&EdgeCubieCube> for EOCoordAll {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_eocoord_all(value) }
    }
}

impl From<&EdgeCubieCube> for EOCoordUD {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_eocoord_ud(value) }
    }
}

impl From<&CubieCube> for EOCoordUD {
    fn from(value: &CubieCube) -> Self {
        EOCoordUD::from(&value.edges)
    }
}

impl From<&EdgeCubieCube> for EOCoordFB {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_eocoord_fb(value) }
    }
}

impl From<&CubieCube> for EOCoordFB {
    fn from(value: &CubieCube) -> Self {
        EOCoordFB::from(&value.edges)
    }
}

impl From<&EdgeCubieCube> for EOCoordLR {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_eocoord_lr(value) }
    }
}

impl From<&CubieCube> for EOCoordLR {
    fn from(value: &CubieCube) -> Self {
        EOCoordLR::from(&value.edges)
    }
}

impl From<&[Edge; 12]> for EOCoordAll {
    fn from(value: &[Edge; 12]) -> Self {
        let mut ud = 0_u16;
        let mut fb = 0_u16;
        let mut lr = 0_u16;

        for i in (0..11).rev() {
            let edge = value[i];
            ud = (ud << 1) | (!edge.oriented_ud as u16);
            fb = (fb << 1) | (!edge.oriented_fb as u16);
            lr = (lr << 1) | (!edge.oriented_rl as u16);
        }

        EOCoordAll(EOCoordUD(ud), EOCoordFB(fb), EOCoordLR(lr))
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_movemask_epi8, _mm_set_epi8, _mm_slli_epi64};

    use crate::coords::eo::{EOCoordAll, EOCoordFB, EOCoordLR, EOCoordUD};
    use crate::cubie::EdgeCubieCube;

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_all(value: &EdgeCubieCube) -> EOCoordAll {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_set_epi8(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                0x0F, 0x0F,
            ),
        );
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        let lr = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordAll(EOCoordUD(ud), EOCoordFB(fb), EOCoordLR(lr))
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_ud(value: &EdgeCubieCube) -> EOCoordUD {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_set_epi8(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                0x0F, 0x0F,
            ),
        );
        let ud = _mm_movemask_epi8(_mm_slli_epi64::<4>(no_db_edge)) as u16;
        EOCoordUD(ud)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_fb(value: &EdgeCubieCube) -> EOCoordFB {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_set_epi8(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                0x0F, 0x0F,
            ),
        );
        let fb = _mm_movemask_epi8(_mm_slli_epi64::<5>(no_db_edge)) as u16;
        EOCoordFB(fb)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub(crate) unsafe fn unsafe_from_eocoord_lr(value: &EdgeCubieCube) -> EOCoordLR {
        //Number of oriented edges is always even, so the last edge can be ignored in the coordinate
        let no_db_edge = _mm_and_si128(
            value.0,
            _mm_set_epi8(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
                0x0F, 0x0F,
            ),
        );
        let rl = _mm_movemask_epi8(_mm_slli_epi64::<6>(no_db_edge)) as u16;
        EOCoordLR(rl)
    }
}