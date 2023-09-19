use std::cmp::Ordering;
use crate::avx2_coord::avx2_coord;
use crate::cube::{Corner, Edge};
use crate::cubie::{CubieCube, CornerCubieCube, EdgeCubieCube};

pub trait Coord<const SIZE: usize>: Into<usize> + Copy + Clone + Eq + PartialEq{
    fn size() -> usize {
        SIZE
    }
    fn val(&self) -> usize;
}

//Edge orientation on the respective axis
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordAll(pub EOCoordUD, pub EOCoordFB, pub EOCoordLR);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordUD(pub u16);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordFB(pub u16);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordLR(pub u16);

//EO without considering edges in the UD slice (because they are already oriented)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EOCoordNoUDSlice(pub u8);

//UD corner orientation
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct COUDCoord(pub u16);

//Corner permutation
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CPCoord(pub u16);

//Edge permutation
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EPCoord(pub u32);

//Coordinate representing the position of edges that belong into the UD slice.
//0 if they are in the slice, they don't have to be in the correct position
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UDSliceUnsortedCoord(pub u16);

//Assuming we already have FB-EO, represents the combination of UDSliceUnsortedCoord and COUDCoord
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DRUDEOFBCoord(u32);

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

impl Coord<128> for EOCoordNoUDSlice {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EOCoordNoUDSlice {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2187> for COUDCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for COUDCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<40320> for CPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for CPCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<479001600> for EPCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for EPCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<495> for UDSliceUnsortedCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for UDSliceUnsortedCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

//TODO this should use 'impl const' once it's stable
const DRUDEOFB_SIZE: usize = 495 * 2187;
impl Coord<DRUDEOFB_SIZE> for DRUDEOFBCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for DRUDEOFBCoord {
    fn into(self) -> usize {
        self.val()
    }
}

impl From<&EdgeCubieCube> for EOCoordAll {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_eocoord_all(value)
        }
    }
}

impl From<&EdgeCubieCube> for EOCoordUD {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_eocoord_ud(value)
        }
    }
}

impl From<&EdgeCubieCube> for EOCoordFB {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_eocoord_fb(value)
        }
    }
}

impl From<&EdgeCubieCube> for EOCoordLR {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_eocoord_lr(value)
        }
    }
}

impl From<&EdgeCubieCube> for EOCoordNoUDSlice {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_eocoord_no_ud_slice(value)
        }
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

impl From<&CornerCubieCube> for COUDCoord {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_cocoord(value)
        }
    }
}

impl From<&[Corner; 8]> for COUDCoord {
    fn from(value: &[Corner; 8]) -> Self {
        let mut co = 0_u16;

        for i in (0..7).rev() {
            co = co * 3 + value[i].orientation as u16;
        }

        COUDCoord(co)
    }
}

impl From<&CornerCubieCube> for CPCoord {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_cpcoord(value)
        }
    }
}

impl From<&[Corner; 8]> for CPCoord {
    fn from(value: &[Corner; 8]) -> Self {
        let mut cp = 0_u16;
        let factorial = [1, 2, 6, 24, 120, 720, 5040];

        for i in 1..8 {
            let mut higher = 0;
            for j in 0..i {
                if value[i].id < value[j].id {
                    higher += 1;
                }
            }
            cp += factorial[i - 1] * higher;
        }
        CPCoord(cp)
    }
}

impl From<&EdgeCubieCube> for EPCoord {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_epcoord(value)
        }
    }
}

impl From<&[Edge; 12]> for EPCoord {
    fn from(value: &[Edge; 12]) -> Self {
        let mut ep = 0_u32;
        let factorial = [1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];

        for i in 1..12 {
            let mut higher = 0;
            for j in 0..i {
                if value[i].id < value[j].id {
                    higher += 1;
                }
            }
            ep += factorial[i - 1] * higher;
        }
        EPCoord(ep)
    }
}

impl From<&EdgeCubieCube> for UDSliceUnsortedCoord {

    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe {
            avx2_coord::unsafe_from_udsliceunsortedcoord(value)
        }
    }
}

impl From<&CubieCube> for DRUDEOFBCoord {
    #[inline]
    fn from(value: &CubieCube) -> Self {
        let ud_slice = UDSliceUnsortedCoord::from(&value.edges).val();
        let co = COUDCoord::from(&value.corners).val();
        let index =
            co * UDSliceUnsortedCoord::size() +
            ud_slice;
        DRUDEOFBCoord(index as u32)
    }
}

pub(crate) const FACTORIAL: [u32; 12] = [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800, 39916800];

pub(crate) const fn b(n: u8, k: u8) -> u8 {
    if n == 0 || n < k {
        return 0;
    }
    (FACTORIAL[n as usize] / FACTORIAL[k as usize] / FACTORIAL[(n - k) as usize]) as u8
}