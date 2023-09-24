#[cfg(all(target_arch = "wasm32", not(target_feature = "avx2")))]
pub mod wasm32_cubie {
    use crate::alignment::AlignedU8;
    use crate::cube::{Axis, Corner, Edge, Face, Turn};
    use crate::cubie::{CornerCubieCube, EdgeCubieCube};
    use std::arch::wasm32;
    use std::arch::wasm32::{
        u16x8_shr, u32x4, u32x4_shl, u32x4_shr, u64x2_extract_lane, u8x16, u8x16_add, u8x16_shl,
        u8x16_shuffle, u8x16_sub, v128, v128_and, v128_andnot, v128_load, v128_or, v128_store,
        v128_xor,
    };

    pub(crate) struct WASM32EdgeCubieCube;

    impl WASM32EdgeCubieCube {
        const VALID_EDGE_MASK_HI: u64 = 0x00000000FFFFFFFF;

        //UB UR UF UL FR FL BR BL DF DR DB DL
        // 0  1  2  3  4  5  6  7  8  9 10 11
        const TURN_EDGE_SHUFFLE: [[v128; 3]; 6] = unsafe {
            [
                [
                    wasm32::u8x16(3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //U
                    wasm32::u8x16(2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //U2
                    wasm32::u8x16(1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //U'
                ],
                [
                    wasm32::u8x16(0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF), //D
                    wasm32::u8x16(0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF), //D2
                    wasm32::u8x16(0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF), //D'
                ],
                [
                    wasm32::u8x16(0, 1, 5, 3, 2, 8, 6, 7, 4, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //F
                    wasm32::u8x16(0, 1, 8, 3, 5, 4, 6, 7, 2, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //F2
                    wasm32::u8x16(0, 1, 4, 3, 8, 2, 6, 7, 5, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //F'
                ],
                [
                    wasm32::u8x16(6, 1, 2, 3, 4, 5, 10, 0, 8, 9, 7, 11, 0xFF, 0xFF, 0xFF, 0xFF), //B
                    wasm32::u8x16(10, 1, 2, 3, 4, 5, 7, 6, 8, 9, 0, 11, 0xFF, 0xFF, 0xFF, 0xFF), //B2
                    wasm32::u8x16(7, 1, 2, 3, 4, 5, 0, 10, 8, 9, 6, 11, 0xFF, 0xFF, 0xFF, 0xFF), //B'
                ],
                [
                    wasm32::u8x16(0, 1, 2, 7, 4, 3, 6, 11, 8, 9, 10, 5, 0xFF, 0xFF, 0xFF, 0xFF), //L
                    wasm32::u8x16(0, 1, 2, 11, 4, 7, 6, 5, 8, 9, 10, 3, 0xFF, 0xFF, 0xFF, 0xFF), //L2
                    wasm32::u8x16(0, 1, 2, 5, 4, 11, 6, 3, 8, 9, 10, 7, 0xFF, 0xFF, 0xFF, 0xFF), //L'
                ],
                [
                    wasm32::u8x16(0, 4, 2, 3, 9, 5, 1, 7, 8, 6, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //R
                    wasm32::u8x16(0, 9, 2, 3, 6, 5, 4, 7, 8, 1, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //R2
                    wasm32::u8x16(0, 6, 2, 3, 1, 5, 9, 7, 8, 4, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF), //R'
                ],
            ]
        };

        const TURN_EO_FLIP: [v128; 6] = unsafe {
            [
                wasm32::u8x16(
                    0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0,
                ), //U
                wasm32::u8x16(
                    0, 0, 0, 0, 0, 0, 0, 0, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0,
                    0, 0,
                ), //D
                wasm32::u8x16(
                    0, 0, 0b00000100, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0, 0,
                    0, 0,
                ), //F
                wasm32::u8x16(
                    0b00000100, 0, 0, 0, 0, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0,
                    0, 0,
                ), //B
                wasm32::u8x16(
                    0, 0, 0, 0b00000010, 0, 0b00000010, 0, 0b00000010, 0, 0, 0, 0b00000010, 0, 0,
                    0, 0,
                ), //L
                wasm32::u8x16(
                    0, 0b00000010, 0, 0, 0b00000010, 0, 0b00000010, 0, 0, 0b00000010, 0, 0, 0, 0,
                    0, 0,
                ), //R
            ]
        };

        const TRANSFORMATION_EP_SHUFFLE: [[v128; 3]; 3] = unsafe {
            [
                [
                    wasm32::u8x16(2, 4, 8, 5, 9, 11, 1, 3, 10, 6, 0, 7, 0xFF, 0xFF, 0xFF, 0xFF), //x
                    wasm32::u8x16(8, 9, 10, 11, 6, 7, 4, 5, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF), //x2
                    wasm32::u8x16(10, 6, 0, 7, 1, 3, 9, 11, 2, 4, 8, 5, 0xFF, 0xFF, 0xFF, 0xFF), //x'
                ],
                [
                    wasm32::u8x16(3, 0, 1, 2, 6, 4, 7, 5, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF), //y
                    wasm32::u8x16(2, 3, 0, 1, 7, 6, 5, 4, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF), //y2
                    wasm32::u8x16(1, 2, 3, 0, 5, 7, 4, 6, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF), //y'
                ],
                [
                    wasm32::u8x16(7, 3, 5, 11, 2, 8, 0, 10, 4, 1, 6, 9, 0xFF, 0xFF, 0xFF, 0xFF), //z
                    wasm32::u8x16(10, 11, 8, 9, 5, 4, 7, 6, 2, 3, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF), //z2
                    wasm32::u8x16(6, 9, 4, 1, 8, 2, 10, 0, 5, 11, 7, 3, 0xFF, 0xFF, 0xFF, 0xFF), //z'
                ],
            ]
        };

        const TRANSFORMATION_EO_MAP: [v128; 3] = unsafe {
            [
                wasm32::u8x16(
                    0b0000, 0xFF, 0b0010, 0xFF, 0b1000, 0xFF, 0b1010, 0xFF, 0b0100, 0xFF, 0b0110,
                    0xFF, 0b1100, 0xFF, 0b1110, 0xFF,
                ), //X
                wasm32::u8x16(
                    0b0000, 0xFF, 0b0100, 0xFF, 0b0010, 0xFF, 0b0110, 0xFF, 0b1000, 0xFF, 0b1100,
                    0xFF, 0b1010, 0xFF, 0b1110, 0xFF,
                ), //X
                wasm32::u8x16(
                    0b0000, 0xFF, 0b1000, 0xFF, 0b0100, 0xFF, 0b1100, 0xFF, 0b0010, 0xFF, 0b1010,
                    0xFF, 0b0110, 0xFF, 0b1110, 0xFF,
                ), //X
            ]
        };

        pub(crate) fn get_edges_raw(cube: &EdgeCubieCube) -> [u64; 2] {
            let low = u64x2_extract_lane::<0>(cube.0);
            let high = u64x2_extract_lane::<1>(cube.0);
            [low, high]
        }

        pub(crate) fn new_solved() -> EdgeCubieCube {
            EdgeCubieCube(u8x16_shl(
                u8x16(0, 0, 0, 0, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0),
                4,
            ))
        }

        pub(crate) fn get_edges(cube: &EdgeCubieCube) -> [Edge; 12] {
            let mut edges = WASM32EdgeCubieCube::get_edges_raw(cube);
            let mut edge_arr = [Edge {
                id: 0,
                oriented_ud: true,
                oriented_fb: true,
                oriented_rl: true,
            }; 12];

            for eid in 0..12 {
                let arr_id = eid / 8;
                let edge = (edges[arr_id] & 0xFF) as u8;
                edges[arr_id] >>= 8;

                let rl = edge & 0b0010 == 0;
                let fb = edge & 0b0100 == 0;
                let ud = edge & 0b1000 == 0;

                edge_arr[eid] = Edge {
                    id: edge >> 4,
                    oriented_ud: ud,
                    oriented_fb: fb,
                    oriented_rl: rl,
                };
            }

            edge_arr
        }

        pub(crate) fn turn(cube: &mut EdgeCubieCube, face: Face, turn_type: Turn) {
            cube.0 = u8x16_shuffle(
                cube.0,
                Self::TURN_EDGE_SHUFFLE[face as usize][turn_type as usize],
            );
            if turn_type != Turn::Half {
                cube.0 = v128_xor(cube.0, Self::TURN_EO_FLIP[face as usize]);
            }
        }

        pub(crate) fn transform(cube: &mut EdgeCubieCube, axis: Axis, turn_type: Turn) {
            let edges_translated = u8x16_shuffle(
                cube.0,
                Self::TRANSFORMATION_EP_SHUFFLE[axis as usize][turn_type as usize],
            );
            let ep = u32x4_shl(v128_and(edges_translated, u8x16set1(0xF0)), 4);
            let eo = v128_and(edges_translated, u8x16set1(0b00001110));
            let ep_translated = u32x4_shl(
                u8x16_shuffle(
                    Self::TRANSFORMATION_EP_SHUFFLE[axis as usize][turn_type.invert() as usize],
                    ep,
                ),
                4,
            );
            let eo = if turn_type != Turn::Half {
                u8x16_shuffle(Self::TRANSFORMATION_EO_MAP[axis], eo)
            } else {
                eo
            };
            cube.0 = v128_or(ep_translated, eo);
        }

        pub(crate) fn invert(cube: &mut EdgeCubieCube) {
            let edge_ids = unsafe {
                let mut a_arr = [0u8; 16];
                v128_store(
                    a_arr.as_mut_ptr() as *mut v128,
                    u32x4_shr(v128_and(cube.0, u8x16set1(0xF0)), 4),
                );
                a_arr
            };
            //This essentially calculates the inverse of u8x16_shuffle(solved_cube.edges, self.edges), same for corners
            let mut edge_shuffle = [0u8; 16];
            for i in 0..12 {
                edge_shuffle[edge_ids[i] as usize] = i as u8;
            }
            let edge_shuffle_mask = unsafe { v128_load(edge_shuffle.as_ptr() as *const v128) };

            //Splice together the edge permutation, and the EO of the edges on the inverse (see niss prediction to see how this works)
            let ep = v128_and(
                u8x16_shuffle(u8x16_shuffle(cube.0, edge_shuffle_mask), edge_shuffle_mask),
                u8x16set1(0xF0),
            );
            let eo_shuffle = u8x16_shuffle(cube.0, u32x4_shr(ep, 4));
            let eo = v128_and(eo_shuffle, u8x16set1(0b1110));

            cube.0 = v128_or(ep, eo);
        }
    }

    pub struct WASM32CornerCubieCube;

    impl WASM32CornerCubieCube {
        const TURN_CORNER_SHUFFLE: [[v128; 3]; 6] = [
            [
                u8x16(
                    3, 0, 1, 2, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //U
                u8x16(
                    2, 3, 0, 1, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //U2
                u8x16(
                    1, 2, 3, 0, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //U'
            ],
            [
                u8x16(
                    0, 1, 2, 3, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //D
                u8x16(
                    0, 1, 2, 3, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //D2
                u8x16(
                    0, 1, 2, 3, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //D'
            ],
            [
                u8x16(
                    0, 1, 3, 4, 5, 2, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F
                u8x16(
                    0, 1, 4, 5, 2, 3, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F2
                u8x16(
                    0, 1, 5, 2, 3, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F'
            ],
            [
                u8x16(
                    1, 6, 2, 3, 4, 5, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //B
                u8x16(
                    6, 7, 2, 3, 4, 5, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //B2
                u8x16(
                    7, 0, 2, 3, 4, 5, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //B'
            ],
            [
                u8x16(
                    7, 1, 2, 0, 3, 5, 6, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //L
                u8x16(
                    4, 1, 2, 7, 0, 5, 6, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //L2
                u8x16(
                    3, 1, 2, 4, 7, 5, 6, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //L'
            ],
            [
                u8x16(
                    0, 2, 5, 3, 4, 6, 1, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //R
                u8x16(
                    0, 5, 6, 3, 4, 1, 2, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //R2
                u8x16(
                    0, 6, 1, 3, 4, 2, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //R'
            ],
        ];

        const TRANSFORMATION_CP_SHUFFLE: [[v128; 3]; 3] = [
            [
                u8x16(
                    3, 2, 5, 4, 7, 6, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //x
                u8x16(
                    4, 5, 6, 7, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //x2
                u8x16(
                    7, 6, 1, 0, 3, 2, 5, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //x'
            ],
            [
                u8x16(
                    3, 0, 1, 2, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //y
                u8x16(
                    2, 3, 0, 1, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //y2
                u8x16(
                    1, 2, 3, 0, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //y'
            ],
            [
                u8x16(
                    7, 0, 3, 4, 5, 2, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F
                u8x16(
                    6, 7, 4, 5, 2, 3, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F2
                u8x16(
                    1, 6, 5, 2, 3, 4, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                ), //F'
            ],
        ];

        //CPOO
        // const CO_MAP: __m128i = u8x16(0b00, 0b01, 0b10, 0xFF, 0b01, 0b00, 0b10, 0xFF, 0b10, 0b01, 0b00, 0xFF, 0b00, 0b01, 0b10, 0xFF); //z

        const TRANSFORMATION_CO_MAP: [v128; 3] = [
            u8x16(
                0b00, 0b01, 0b10, 0xFF, 0b01, 0b10, 0b00, 0xFF, 0b10, 0b00, 0b01, 0xFF, 0b00, 0b01,
                0b10, 0xFF,
            ), //z
            u8x16(
                0b00, 0b01, 0b10, 0xFF, 0b00, 0b01, 0b10, 0xFF, 0b00, 0b01, 0b10, 0xFF, 0b00, 0b01,
                0b10, 0xFF,
            ), //y
            u8x16(
                0b00, 0b01, 0b10, 0xFF, 0b10, 0b00, 0b01, 0xFF, 0b01, 0b10, 0b00, 0xFF, 0b00, 0b01,
                0b10, 0xFF,
            ), //x
        ];

        const CO_OVERFLOW_MASK: v128 = u8x16(
            0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100,
            0b00000100, 0, 0, 0, 0, 0, 0, 0, 0,
        );

        const TURN_CO_CHANGE: [v128; 6] = [
            u8x16(1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0), //U
            u8x16(1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0), //D
            u8x16(1, 1, 2, 3, 2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0), //F
            u8x16(2, 3, 1, 1, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0), //B
            u8x16(3, 1, 1, 2, 3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0), //L
            u8x16(1, 2, 3, 1, 1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0), //R
        ];

        #[inline]
        pub(crate) fn new_solved() -> CornerCubieCube {
            CornerCubieCube(u8x16_shl(
                u8x16(0, 0, 0, 0, 0, 0, 0, 0, 7, 6, 5, 4, 3, 2, 1, 0),
                5,
            ))
        }

        #[inline]
        pub(crate) fn get_corners_raw(cube: &CornerCubieCube) -> u64 {
            u64x2_extract_lane::<0>(cube.0)
        }

        #[inline]
        pub(crate) fn get_corners(cube: &CornerCubieCube) -> [Corner; 8] {
            let mut corner_bits = WASM32CornerCubieCube::get_corners_raw(cube);
            let mut corner_arr = [Corner {
                id: 0,
                orientation: 0,
            }; 8];
            for cid in 0..8 {
                let corner = (corner_bits & 0xFF) as u8;
                corner_bits >>= 8;
                corner_arr[cid] = Corner {
                    id: corner >> 5,
                    orientation: corner & 0x7,
                };
            }
            corner_arr
        }

        #[inline]
        pub(crate) fn turn(cube: &mut CornerCubieCube, face: Face, turn_type: Turn) {
            cube.0 = u8x16_shuffle(
                cube.0,
                Self::TURN_CORNER_SHUFFLE[face as usize][turn_type as usize],
            );
            if turn_type != Turn::Half {
                //Valid COs are 00, 01, 10. When we move, we don't add 0, 1, 2 (no change, clockwise, counter-clockwise), but we add 1, 2, 3 to force overflowing into the next bit.
                //This code either subtracts 1 if there is no overflow (because we added 1 too much before), or 4, because this gives us the original addition mod 3.
                let corners_tmp = u8x16_add(cube.0, Self::TURN_CO_CHANGE[face as usize]);
                let overflow_bits = v128_and(corners_tmp, Self::CO_OVERFLOW_MASK);
                let not_overflow = u16x8_shr(v128_andnot(corners_tmp, Self::CO_OVERFLOW_MASK), 2);
                let overflow_sub = v128_or(overflow_bits, not_overflow);
                cube.0 = u8x16_sub(corners_tmp, overflow_sub);
            }
        }

        #[inline]
        pub(crate) fn transform(cube: &mut CornerCubieCube, axis: Axis, turn_type: Turn) {
            let corners_translated = u8x16_shuffle(
                cube.0,
                Self::TRANSFORMATION_CP_SHUFFLE[axis as usize][turn_type as usize],
            );
            let cp = u32x4_shr(v128_and(corners_translated, u8x16set1(0b11100000)), 5);
            let co = v128_and(corners_translated, u8x16set1(0b00000011));
            let cp_translated = u32x4_shl(
                u8x16_shuffle(
                    Self::TRANSFORMATION_CP_SHUFFLE[axis as usize][turn_type.invert() as usize],
                    cp,
                ),
                5,
            );
            let co = if turn_type != Turn::Half {
                let corner_orbit_id = v128_and(cp_translated, u8x16set1(0b00100000));
                //We want 4 bits. The lowest two are for the corner CO, the third tells us which orbit the corner belongs to, and the fourth is which orbit the corner is in.
                //Changing the CO only depends on the axis, corner orbit and previous UD-CO, so we can just use a lookup table to do this in a simple way
                let co_id = v128_or(u32x4_shr(corner_orbit_id, 3), co);
                let co_id = v128_or(
                    co_id,
                    u8x16(
                        0, 0, 0, 0, 0, 0, 0, 0, 0b1000, 0, 0b1000, 0, 0b1000, 0, 0b1000, 0,
                    ),
                );
                u8x16_shuffle(Self::TRANSFORMATION_CO_MAP[axis], co_id)
            } else {
                co
            };
            cube.0 = v128_or(cp_translated, co);
        }

        #[inline]
        pub(crate) fn invert(cube: &mut CornerCubieCube) {
            let corner_ids =
                (u64x2_extract_lane::<0>(u32x4_shr(v128_and(cube.0, u8x16set1(0xE0)), 5)))
                    .to_le_bytes();

            let mut corner_shuffle = corner_ids.clone();
            for i in 0..8 {
                corner_shuffle[corner_ids[i] as usize] = i as u8;
            }
            let corner_shuffle_mask = unsafe { v128_load(corner_shuffle.as_ptr() as *const v128) };

            //Splice together the corner permutation, and the CO of the corners on the inverse (see niss prediction to see how this works)
            //Also switch CO 1 <-> 2,  CO 0 stays the same
            let cp = v128_and(
                u8x16_shuffle(
                    u8x16_shuffle(cube.0, corner_shuffle_mask),
                    corner_shuffle_mask,
                ),
                u8x16set1(0b11100000),
            );
            let co_shuffle = u8x16_shuffle(cube.0, u32x4_shr(cp, 5));
            let tmp = v128_and(u8x16_add(co_shuffle, u8x16set1(1)), u8x16set1(2));
            let co_flip_mask = v128_or(tmp, u32x4_shr(tmp, 1));
            let co = v128_and(v128_xor(co_shuffle, co_flip_mask), u8x16set1(7));

            cube.0 = v128_or(cp, co);
        }
    }

    fn u8x16set1(a: u8) -> v128 {
        u8x16(a, a, a, a, a, a, a, a, a, a, a, a, a, a, a, a)
    }
}
