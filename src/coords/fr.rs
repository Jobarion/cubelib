use crate::coords::coord::Coord;
use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FREdgesCoord(pub(crate) u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRCPOrbitCoord(pub(crate) u8);

//Coordinate representing the orbit parity state in floppy reduction
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FROrbitParityCoord(pub(crate) bool);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRUDNoSliceCoord(pub(crate) u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRSliceEdgesCoord(pub(crate) u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FRUDWithSliceCoord(pub(crate) u16);

impl Coord<64> for FREdgesCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FREdgesCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<4> for FRCPOrbitCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRCPOrbitCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl Coord<2> for FROrbitParityCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FROrbitParityCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

pub const FRUD_NO_SLICE_SIZE: usize = 2 * 4 * 64;
impl Coord<FRUD_NO_SLICE_SIZE> for FRUDNoSliceCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRUDNoSliceCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

//Actually only 12 possible values, but that'd be harder to compute
impl Coord<16> for FRSliceEdgesCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRSliceEdgesCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

pub const FRUD_WITH_SLICE_SIZE: usize = FRUD_NO_SLICE_SIZE * 16;
impl Coord<FRUD_WITH_SLICE_SIZE> for FRUDWithSliceCoord {
    fn val(&self) -> usize {
        self.0 as usize
    }
}

impl Into<usize> for FRUDWithSliceCoord {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl From<&EdgeCubieCube> for FREdgesCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &EdgeCubieCube) -> Self {
        unsafe { avx2::unsafe_from_fr_edges_coord(value) }
    }
}

impl From<&CornerCubieCube> for FRCPOrbitCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CornerCubieCube) -> Self {
        unsafe { avx2::unsafe_from_fr_cp_coord(value) }
    }
}

impl From<&CubieCube> for FROrbitParityCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CubieCube) -> Self {
        unsafe { avx2::unsafe_from_fr_parity_coord(value) }
    }
}

impl From<&CubieCube> for FRSliceEdgesCoord {
    #[inline]
    #[cfg(target_feature = "avx2")]
    fn from(value: &CubieCube) -> Self {
        unsafe { avx2::unsafe_from_fr_slice_coord(value) }
    }
}

impl From<&CubieCube> for FRUDNoSliceCoord {
    fn from(value: &CubieCube) -> Self {
        let edges = FREdgesCoord::from(&value.edges).val();
        let orbit_cp = FRCPOrbitCoord::from(&value.corners).val();
        let orbit_parity = FROrbitParityCoord::from(value).val();

        // println!("{:?}", edges);
        // println!("{:?}", orbit_cp);
        // println!("{:?}", orbit_parity);

        let coord = orbit_parity +
            orbit_cp * FROrbitParityCoord::size() +
            edges * FROrbitParityCoord::size() * FRCPOrbitCoord::size();
        FRUDNoSliceCoord(coord as u16)
    }
}

impl From<&CubieCube> for FRUDWithSliceCoord {
    fn from(value: &CubieCube) -> Self {
        let no_slice = FRUDNoSliceCoord::from(value).val();
        let slice = FRSliceEdgesCoord::from(value).val();

        let coord = slice +
            no_slice * FRSliceEdgesCoord::size();
        FRUDWithSliceCoord(coord as u16)
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use std::arch::x86_64::{_mm_and_si128, _mm_castpd_si128, _mm_castsi128_pd, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_extract_epi16, _mm_movemask_epi8, _mm_or_si128, _mm_permute_pd, _mm_sad_epu8, _mm_set1_epi32, _mm_set1_epi8, _mm_set_epi8, _mm_shuffle_epi8, _mm_srli_epi32, _mm_xor_si128};
    use crate::coords::fr::{FRCPOrbitCoord, FREdgesCoord, FROrbitParityCoord, FRSliceEdgesCoord};
    use crate::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_slice_coord(cube: &CubieCube) -> FRSliceEdgesCoord {
        let corner_edge_mapping = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0b01110000, 0b01100000, 0b01000000, 0b01010000, 0b01010000, 0b01000000, 0b01100000, 0b01110000);

        // println!("{:?}", corner_edge_mapping);
        // println!("{:?}", _mm_and_si128(_mm_srli_epi32::<5>(cube.corners.0), _mm_set1_epi8(0x0F)));

        let associated_edges = _mm_shuffle_epi8(corner_edge_mapping, _mm_and_si128(_mm_srli_epi32::<5>(cube.corners.0), _mm_set1_epi8(0x0F)));

        // println!("{:?}", associated_edges);

        let correct_bl_edge_position = _mm_cmpeq_epi8(cube.edges.0, _mm_shuffle_epi8(associated_edges, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, -1, -1, -1, -1)));
        let correct_br_edge_position = _mm_cmpeq_epi8(cube.edges.0, _mm_shuffle_epi8(associated_edges, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 1, 1, 1, 1, -1, -1, -1, -1)));

        // println!("Edges: {:?}", cube.edges.0);
        // println!("{:?}", correct_bl_edge_position);
        // println!("{:?}", correct_br_edge_position);


        let bl_edge_values = _mm_and_si128(correct_bl_edge_position, _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0, 0));
        let br_edge_values = _mm_and_si128(correct_br_edge_position, _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 8, 12, 0, 0, 0, 0));

        let edge_coord = _mm_sad_epu8(_mm_or_si128(bl_edge_values, br_edge_values), _mm_set1_epi8(0));
        let coord = _mm_extract_epi16::<0>(edge_coord) as u8;

        FRSliceEdgesCoord(coord)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_edges_coord(cube: &EdgeCubieCube) -> FREdgesCoord {
        let relevant_edges = _mm_shuffle_epi8(cube.0, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 9, 8, 3, 2, 1, 0));
        let ids = _mm_and_si128(_mm_srli_epi32::<4>(relevant_edges), _mm_set1_epi8(0x0F));
        let fr_colors = _mm_shuffle_epi8(_mm_set_epi8(-1, -1, -1, -1, 0, 0, 1, 1, -1, -1, -1, -1, 0, 1, 1, 0), ids);
        let incorrect = _mm_cmpeq_epi8(fr_colors, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 1, 0, 0, 1));
        let coord = _mm_movemask_epi8(incorrect) as u8;
        FREdgesCoord(coord)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_cp_coord(cube: &CornerCubieCube) -> FRCPOrbitCoord {
        let opposites = _mm_and_si128(_mm_xor_si128(cube.0, _mm_set1_epi32(-1)), _mm_set1_epi8(0b11100000_u8 as i8));
        let all_ubl_opposite = _mm_shuffle_epi8(opposites, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0));
        let opposite_position = _mm_cmpeq_epi8(cube.0, all_ubl_opposite);
        let position_values = _mm_and_si128(opposite_position, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, 0, -1, 3, -1, 2, -1, 1, -1));
        let coord = _mm_extract_epi16::<0>(_mm_sad_epu8(position_values, _mm_set1_epi8(0))) as u8;
        FRCPOrbitCoord(coord)
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn unsafe_from_fr_parity_coord(cube: &CubieCube) -> FROrbitParityCoord {
        let orbit_corners = _mm_shuffle_epi8(cube.corners.0, _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 6, 6, 6, 4, 4, 2));
        let slice_edges = _mm_shuffle_epi8(cube.edges.0, _mm_set_epi8(-1, -1, 7, 7, 7, 6, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1));

        // println!("{:?}", slice_edges);
        let cmp_corners = _mm_shuffle_epi8(
            cube.corners.0,
            _mm_set_epi8(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 4, 2, 0, 2, 0, 0),
        );
        let cmp_edges = _mm_shuffle_epi8(
            cube.edges.0,
            _mm_set_epi8(-1, -1, 6, 5, 4, 5, 4, 4, -1, -1, -1, -1, -1, -1, -1, -1),
        );
        // println!("{:?}", cmp_edges);
        let cmp0 = _mm_or_si128(orbit_corners, slice_edges);
        let cmp1 = _mm_or_si128(cmp_edges, cmp_corners);
        let higher_left = _mm_and_si128(
            _mm_cmpgt_epi8(cmp0, cmp1),
            _mm_set1_epi8(1),
        );
        // println!("{:?}", higher_left);
        let added = _mm_sad_epu8(higher_left, _mm_set1_epi8(0));
        // println!("{:?}", added);
        let added = _mm_xor_si128(added, _mm_castpd_si128(_mm_permute_pd::<0b11>(_mm_castsi128_pd(added))));
        // println!("{:?}", added);
        let parity = _mm_extract_epi16::<0>(added) & 1;
        FROrbitParityCoord(parity == 1)
    }
}