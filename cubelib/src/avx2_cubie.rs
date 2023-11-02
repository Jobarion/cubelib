#[cfg(target_feature = "avx2")]
pub mod avx2_cubie {
    use std::arch::x86_64::{
        __m128i, _mm_add_epi8, _mm_and_si128, _mm_andnot_si128, _mm_extract_epi64, _mm_load_si128,
        _mm_loadl_epi64, _mm_or_si128, _mm_set1_epi8, _mm_setr_epi8,
        _mm_shuffle_epi8, _mm_slli_epi32, _mm_slli_epi64, _mm_srli_epi16, _mm_srli_epi32,
        _mm_store_si128, _mm_sub_epi8, _mm_xor_si128,
    };

    use crate::alignment::avx2::C;
    use crate::alignment::{AlignedU64, AlignedU8};
    use crate::cube::{Axis, Corner, Edge, Face, Turn};
    use crate::cubie::CornerCubieCube;
    use crate::cubie::EdgeCubieCube;

    pub(crate) struct AVX2EdgeCubieCube;

    impl AVX2EdgeCubieCube {
        const VALID_EDGE_MASK_HI: u64 = 0x00000000FFFFFFFF;

        //UB UR UF UL FR FL BR BL DF DR DB DL
        // 0  1  2  3  4  5  6  7  8  9 10 11
        const TURN_EDGE_SHUFFLE: [[__m128i; 3]; 6] = [
            [
                unsafe {
                    C {
                        a_u8: [3, 0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //U
                unsafe {
                    C {
                        a_u8: [2, 3, 0, 1, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //U2
                unsafe {
                    C {
                        a_u8: [1, 2, 3, 0, 4, 5, 6, 7, 8, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //U'
            ],
            [
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //D
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //D2
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //D'
            ],
            [
                unsafe {
                    C {
                        a_u8: [0, 1, 5, 3, 2, 8, 6, 7, 4, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //F
                unsafe {
                    C {
                        a_u8: [0, 1, 8, 3, 5, 4, 6, 7, 2, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //F2
                unsafe {
                    C {
                        a_u8: [0, 1, 4, 3, 8, 2, 6, 7, 5, 9, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //F'
            ],
            [
                unsafe {
                    C {
                        a_u8: [6, 1, 2, 3, 4, 5, 10, 0, 8, 9, 7, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //B
                unsafe {
                    C {
                        a_u8: [10, 1, 2, 3, 4, 5, 7, 6, 8, 9, 0, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //B2
                unsafe {
                    C {
                        a_u8: [7, 1, 2, 3, 4, 5, 0, 10, 8, 9, 6, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //B'
            ],
            [
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 7, 4, 3, 6, 11, 8, 9, 10, 5, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //L
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 11, 4, 7, 6, 5, 8, 9, 10, 3, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //L2
                unsafe {
                    C {
                        a_u8: [0, 1, 2, 5, 4, 11, 6, 3, 8, 9, 10, 7, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //L'
            ],
            [
                unsafe {
                    C {
                        a_u8: [0, 4, 2, 3, 9, 5, 1, 7, 8, 6, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //R
                unsafe {
                    C {
                        a_u8: [0, 9, 2, 3, 6, 5, 4, 7, 8, 1, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //R2
                unsafe {
                    C {
                        a_u8: [0, 6, 2, 3, 1, 5, 9, 7, 8, 4, 10, 11, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //R'
            ],
        ];

        const TURN_EO_FLIP: [__m128i; 6] = [
            unsafe {
                C {
                    a_u8: [
                        0b00001000, 0b00001000, 0b00001000, 0b00001000, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //U
            unsafe {
                C {
                    a_u8: [
                        0, 0, 0, 0, 0, 0, 0, 0, 0b00001000, 0b00001000, 0b00001000, 0b00001000, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //D
            unsafe {
                C {
                    a_u8: [
                        0, 0, 0b00000100, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0, 0, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //F
            unsafe {
                C {
                    a_u8: [
                        0b00000100, 0, 0, 0, 0, 0, 0b00000100, 0b00000100, 0, 0, 0b00000100, 0, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //B
            unsafe {
                C {
                    a_u8: [
                        0, 0, 0, 0b00000010, 0, 0b00000010, 0, 0b00000010, 0, 0, 0, 0b00000010, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //L
            unsafe {
                C {
                    a_u8: [
                        0, 0b00000010, 0, 0, 0b00000010, 0, 0b00000010, 0, 0, 0b00000010, 0, 0, 0,
                        0, 0, 0,
                    ],
                }
                .a
            }, //R
        ];

        const TRANSFORMATION_EP_SHUFFLE: [[__m128i; 3]; 3] = [
            [
                unsafe {
                    C {
                        a_u8: [2, 4, 8, 5, 9, 11, 1, 3, 10, 6, 0, 7, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //x
                unsafe {
                    C {
                        a_u8: [8, 9, 10, 11, 6, 7, 4, 5, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //x2
                unsafe {
                    C {
                        a_u8: [10, 6, 0, 7, 1, 3, 9, 11, 2, 4, 8, 5, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //x'
            ],
            [
                unsafe {
                    C {
                        a_u8: [3, 0, 1, 2, 6, 4, 7, 5, 9, 10, 11, 8, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //y
                unsafe {
                    C {
                        a_u8: [2, 3, 0, 1, 7, 6, 5, 4, 10, 11, 8, 9, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //y2
                unsafe {
                    C {
                        a_u8: [1, 2, 3, 0, 5, 7, 4, 6, 11, 8, 9, 10, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //y'
            ],
            [
                unsafe {
                    C {
                        a_u8: [7, 3, 5, 11, 2, 8, 0, 10, 4, 1, 6, 9, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //z
                unsafe {
                    C {
                        a_u8: [10, 11, 8, 9, 5, 4, 7, 6, 2, 3, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //z2
                unsafe {
                    C {
                        a_u8: [6, 9, 4, 1, 8, 2, 10, 0, 5, 11, 7, 3, 0xFF, 0xFF, 0xFF, 0xFF],
                    }
                    .a
                }, //z'
            ],
        ];

        const TRANSFORMATION_EO_MAP: [__m128i; 3] = [
            unsafe {
                C {
                    a_u8: [
                        0b0000, 0xFF, 0b0010, 0xFF, 0b1000, 0xFF, 0b1010, 0xFF, 0b0100, 0xFF,
                        0b0110, 0xFF, 0b1100, 0xFF, 0b1110, 0xFF,
                    ],
                }
                .a
            }, //X
            unsafe {
                C {
                    a_u8: [
                        0b0000, 0xFF, 0b0100, 0xFF, 0b0010, 0xFF, 0b0110, 0xFF, 0b1000, 0xFF,
                        0b1100, 0xFF, 0b1010, 0xFF, 0b1110, 0xFF,
                    ],
                }
                .a
            }, //X
            unsafe {
                C {
                    a_u8: [
                        0b0000, 0xFF, 0b1000, 0xFF, 0b0100, 0xFF, 0b1100, 0xFF, 0b0010, 0xFF,
                        0b1010, 0xFF, 0b0110, 0xFF, 0b1110, 0xFF,
                    ],
                }
                .a
            }, //X
        ];

        #[target_feature(enable = "avx2")]
        pub(crate) unsafe fn unsafe_get_edges_raw(cube: &EdgeCubieCube) -> [u64; 2] {
            let mut a_arr = AlignedU64([0u64; 2]).0;
            _mm_store_si128(a_arr.as_mut_ptr() as *mut __m128i, cube.0);
            a_arr[1] &= Self::VALID_EDGE_MASK_HI;
            a_arr
        }

        #[target_feature(enable = "avx2")]
        pub(crate) unsafe fn unsafe_new_solved() -> EdgeCubieCube {
            EdgeCubieCube(unsafe {
                _mm_slli_epi64::<4>(_mm_setr_epi8( 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 0, 0,
                    0))
            })
        }

        #[target_feature(enable = "avx2")]
        pub unsafe fn unsafe_get_edges(cube: &EdgeCubieCube) -> [Edge; 12] {
            let mut edges = unsafe {
                let mut a_arr = AlignedU64([0u64; 2]).0;
                _mm_store_si128(a_arr.as_mut_ptr() as *mut __m128i, cube.0);
                a_arr
            };
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

        #[target_feature(enable = "avx2")]
        pub(crate) unsafe fn unsafe_turn(cube: &mut EdgeCubieCube, face: Face, turn_type: Turn) {
            cube.0 = _mm_shuffle_epi8(
                cube.0,
                Self::TURN_EDGE_SHUFFLE[face as usize][turn_type as usize],
            );
            if turn_type != Turn::Half {
                cube.0 = _mm_xor_si128(cube.0, Self::TURN_EO_FLIP[face as usize]);
            }
        }

        #[target_feature(enable = "avx2")]
        pub(crate) unsafe fn unsafe_transform(
            cube: &mut EdgeCubieCube,
            axis: Axis,
            turn_type: Turn,
        ) {
            let edges_translated = _mm_shuffle_epi8(
                cube.0,
                Self::TRANSFORMATION_EP_SHUFFLE[axis as usize][turn_type as usize],
            );
            let ep = _mm_srli_epi32::<4>(_mm_and_si128(
                edges_translated,
                _mm_set1_epi8(0xF0_u8 as i8),
            ));
            let eo = _mm_and_si128(edges_translated, _mm_set1_epi8(0b00001110));
            let ep_translated = _mm_slli_epi32::<4>(_mm_shuffle_epi8(
                Self::TRANSFORMATION_EP_SHUFFLE[axis as usize][turn_type.invert() as usize],
                ep,
            ));
            let eo = if turn_type != Turn::Half {
                _mm_shuffle_epi8(Self::TRANSFORMATION_EO_MAP[axis], eo)
            } else {
                eo
            };
            cube.0 = _mm_or_si128(ep_translated, eo);
        }

        // TODO[perf]
        // We could speed this up by a factor of 10 if changed the cube representation to
        //__m256(normal, inverse) and just swapped the hi and lo parts.
        //Applying turns should be just as quick as before and would just require other masks
        //Right now unsafe_invert runs at about 70m ops/s so that's probably never going to be
        // a bottleneck anyways.
        #[target_feature(enable = "avx2")]
        pub(crate) unsafe fn unsafe_invert(cube: &mut EdgeCubieCube) {
            let edge_ids = unsafe {
                let mut a_arr = AlignedU8([0u8; 16]);
                _mm_store_si128(
                    a_arr.0.as_mut_ptr() as *mut __m128i,
                    _mm_srli_epi32::<4>(_mm_and_si128(cube.0, _mm_set1_epi8(0xF0_u8 as i8))),
                );
                a_arr
            };
            //This essentially calculates the inverse of _mm_shuffle_epi8(solved_cube.edges, self.edges), same for corners
            let mut edge_shuffle = AlignedU8([0u8; 16]);
            let edges = edge_ids.0;
            for i in 0..12 {
                edge_shuffle.0[edges[i] as usize] = i as u8;
            }
            let edge_shuffle_mask = _mm_load_si128(edge_shuffle.0.as_ptr() as *const __m128i);

            //Splice together the edge permutation, and the EO of the edges on the inverse (see niss prediction to see how this works)
            let ep = _mm_and_si128(
                _mm_shuffle_epi8(
                    _mm_shuffle_epi8(cube.0, edge_shuffle_mask),
                    edge_shuffle_mask,
                ),
                _mm_set1_epi8(0xF0_u8 as i8),
            );
            let eo_shuffle = _mm_shuffle_epi8(cube.0, _mm_srli_epi32::<4>(ep));
            let eo = _mm_and_si128(eo_shuffle, _mm_set1_epi8(0b1110));

            cube.0 = _mm_or_si128(ep, eo);
        }
    }

    pub struct AVX2CornerCubieCube;

    impl AVX2CornerCubieCube {
        const TURN_CORNER_SHUFFLE: [[__m128i; 3]; 6] = [
            [
                unsafe {
                    C {
                        a_u8: [
                            3, 0, 1, 2, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //U
                unsafe {
                    C {
                        a_u8: [
                            2, 3, 0, 1, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //U2
                unsafe {
                    C {
                        a_u8: [
                            1, 2, 3, 0, 4, 5, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //U'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 2, 3, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //D
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 2, 3, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //D2
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 2, 3, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //D'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 3, 4, 5, 2, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 4, 5, 2, 3, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F2
                unsafe {
                    C {
                        a_u8: [
                            0, 1, 5, 2, 3, 4, 6, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            1, 6, 2, 3, 4, 5, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //B
                unsafe {
                    C {
                        a_u8: [
                            6, 7, 2, 3, 4, 5, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //B2
                unsafe {
                    C {
                        a_u8: [
                            7, 0, 2, 3, 4, 5, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //B'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            7, 1, 2, 0, 3, 5, 6, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //L
                unsafe {
                    C {
                        a_u8: [
                            4, 1, 2, 7, 0, 5, 6, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //L2
                unsafe {
                    C {
                        a_u8: [
                            3, 1, 2, 4, 7, 5, 6, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //L'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            0, 2, 5, 3, 4, 6, 1, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //R
                unsafe {
                    C {
                        a_u8: [
                            0, 5, 6, 3, 4, 1, 2, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //R2
                unsafe {
                    C {
                        a_u8: [
                            0, 6, 1, 3, 4, 2, 5, 7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //R'
            ],
        ];

        pub(crate) const TRANSFORMATION_CP_SHUFFLE: [[__m128i; 3]; 3] = [
            [
                unsafe {
                    C {
                        a_u8: [
                            3, 2, 5, 4, 7, 6, 1, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //x
                unsafe {
                    C {
                        a_u8: [
                            4, 5, 6, 7, 0, 1, 2, 3, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //x2
                unsafe {
                    C {
                        a_u8: [
                            7, 6, 1, 0, 3, 2, 5, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //x'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            3, 0, 1, 2, 5, 6, 7, 4, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //y
                unsafe {
                    C {
                        a_u8: [
                            2, 3, 0, 1, 6, 7, 4, 5, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //y2
                unsafe {
                    C {
                        a_u8: [
                            1, 2, 3, 0, 7, 4, 5, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //y'
            ],
            [
                unsafe {
                    C {
                        a_u8: [
                            7, 0, 3, 4, 5, 2, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F
                unsafe {
                    C {
                        a_u8: [
                            6, 7, 4, 5, 2, 3, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F2
                unsafe {
                    C {
                        a_u8: [
                            1, 6, 5, 2, 3, 4, 7, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                        ],
                    }
                    .a
                }, //F'
            ],
        ];

        //CPOO
        // const CO_MAP: __m128i = unsafe { C { a_u8: [0b00, 0b01, 0b10, 0xFF, 0b01, 0b00, 0b10, 0xFF, 0b10, 0b01, 0b00, 0xFF, 0b00, 0b01, 0b10, 0xFF] }.a }; //z

        const TRANSFORMATION_CO_MAP: [__m128i; 3] = [
            unsafe {
                C {
                    a_u8: [
                        0b00, 0b01, 0b10, 0xFF, 0b01, 0b10, 0b00, 0xFF, 0b10, 0b00, 0b01, 0xFF,
                        0b00, 0b01, 0b10, 0xFF,
                    ],
                }
                .a
            }, //z
            unsafe {
                C {
                    a_u8: [
                        0b00, 0b01, 0b10, 0xFF, 0b00, 0b01, 0b10, 0xFF, 0b00, 0b01, 0b10, 0xFF,
                        0b00, 0b01, 0b10, 0xFF,
                    ],
                }
                .a
            }, //y
            unsafe {
                C {
                    a_u8: [
                        0b00, 0b01, 0b10, 0xFF, 0b10, 0b00, 0b01, 0xFF, 0b01, 0b10, 0b00, 0xFF,
                        0b00, 0b01, 0b10, 0xFF,
                    ],
                }
                .a
            }, //x
        ];

        const CO_OVERFLOW_MASK: __m128i = unsafe {
            C {
                a_u8: [
                    0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100, 0b00000100,
                    0b00000100, 0b00000100, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
            }
            .a
        };

        const TURN_CO_CHANGE: [__m128i; 6] = [
            unsafe {
                C {
                    a_u8: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //U
            unsafe {
                C {
                    a_u8: [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //D
            unsafe {
                C {
                    a_u8: [1, 1, 2, 3, 2, 3, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //F
            unsafe {
                C {
                    a_u8: [2, 3, 1, 1, 1, 1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //B
            unsafe {
                C {
                    a_u8: [3, 1, 1, 2, 3, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //L
            unsafe {
                C {
                    a_u8: [1, 2, 3, 1, 1, 2, 3, 1, 0, 0, 0, 0, 0, 0, 0, 0],
                }
                .a
            }, //R
        ];

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_new_solved() -> CornerCubieCube {
            CornerCubieCube(unsafe {
                _mm_slli_epi64::<5>(_mm_setr_epi8( 0, 1, 2, 3, 4, 5, 6, 7, 0, 0, 0, 0, 0, 0, 0,0))
            })
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_get_corners_raw(cube: &CornerCubieCube) -> u64 {
            _mm_extract_epi64::<0>(cube.0) as u64
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_get_corners(cube: &CornerCubieCube) -> [Corner; 8] {
            let mut corner_bits = _mm_extract_epi64::<0>(cube.0) as u64;
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

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_turn(cube: &mut CornerCubieCube, face: Face, turn_type: Turn) {
            cube.0 = _mm_shuffle_epi8(
                cube.0,
                Self::TURN_CORNER_SHUFFLE[face as usize][turn_type as usize],
            );
            if turn_type != Turn::Half {
                //Valid COs are 00, 01, 10. When we move, we don't add 0, 1, 2 (no change, clockwise, counter-clockwise), but we add 1, 2, 3 to force overflowing into the next bit.
                //This code either subtracts 1 if there is no overflow (because we added 1 too much before), or 4, because this gives us the original addition mod 3.
                let corners_tmp = _mm_add_epi8(cube.0, Self::TURN_CO_CHANGE[face as usize]);
                let overflow_bits = _mm_and_si128(corners_tmp, Self::CO_OVERFLOW_MASK);
                let not_overflow =
                    _mm_srli_epi16::<2>(_mm_andnot_si128(corners_tmp, Self::CO_OVERFLOW_MASK));
                let overflow_sub = _mm_or_si128(overflow_bits, not_overflow);
                cube.0 = _mm_sub_epi8(corners_tmp, overflow_sub);
            }
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_transform(
            cube: &mut CornerCubieCube,
            axis: Axis,
            turn_type: Turn,
        ) {
            let corners_translated = _mm_shuffle_epi8(
                cube.0,
                Self::TRANSFORMATION_CP_SHUFFLE[axis as usize][turn_type as usize],
            );
            let cp = _mm_srli_epi32::<5>(_mm_and_si128(
                corners_translated,
                _mm_set1_epi8(0b11100000_u8 as i8),
            ));
            let co = _mm_and_si128(corners_translated, _mm_set1_epi8(0b00000011));
            let cp_translated = _mm_slli_epi32::<5>(_mm_shuffle_epi8(
                Self::TRANSFORMATION_CP_SHUFFLE[axis as usize][turn_type.invert() as usize],
                cp,
            ));
            let co = if turn_type != Turn::Half {
                let corner_orbit_id = _mm_and_si128(cp_translated, _mm_set1_epi8(0b00100000));
                //We want 4 bits. The lowest two are for the corner CO, the third tells us which orbit the corner belongs to, and the fourth is which orbit the corner is in.
                //Changing the CO only depends on the axis, corner orbit and previous UD-CO, so we can just use a lookup table to do this in a simple way
                let co_id = _mm_or_si128(_mm_srli_epi32::<3>(corner_orbit_id), co);
                let co_id = _mm_or_si128(
                    co_id,
                    _mm_setr_epi8( 0, 0b1000, 0, 0b1000, 0, 0b1000, 0, 0b1000, 0, 0, 0, 0, 0, 0, 0,
                        0),
                );
                _mm_shuffle_epi8(Self::TRANSFORMATION_CO_MAP[axis], co_id)
            } else {
                co
            };
            cube.0 = _mm_or_si128(cp_translated, co);
        }

        #[target_feature(enable = "avx2")]
        #[inline]
        pub(crate) unsafe fn unsafe_invert(cube: &mut CornerCubieCube) {
            let corner_ids = unsafe {
                (_mm_extract_epi64::<0>(_mm_srli_epi32::<5>(_mm_and_si128(
                    cube.0,
                    _mm_set1_epi8(0xE0_u8 as i8),
                ))) as u64)
                    .to_le_bytes()
            };

            let mut corner_shuffle = corner_ids.clone();
            for i in 0..8 {
                corner_shuffle[corner_ids[i] as usize] = i as u8;
            }
            let corner_shuffle_mask = _mm_loadl_epi64(corner_shuffle.as_ptr() as *const __m128i);

            //Splice together the corner permutation, and the CO of the corners on the inverse (see niss prediction to see how this works)
            //Also switch CO 1 <-> 2,  CO 0 stays the same
            let cp = _mm_and_si128(
                _mm_shuffle_epi8(
                    _mm_shuffle_epi8(cube.0, corner_shuffle_mask),
                    corner_shuffle_mask,
                ),
                _mm_set1_epi8(0b11100000_u8 as i8),
            );
            let co_shuffle = _mm_shuffle_epi8(cube.0, _mm_srli_epi32::<5>(cp));
            let tmp = _mm_and_si128(
                _mm_add_epi8(
                    co_shuffle,
                    _mm_setr_epi8( 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0,0),
                ),
                _mm_set1_epi8(2),
            );
            let co_flip_mask = _mm_or_si128(tmp, _mm_srli_epi32::<1>(tmp));
            let co = _mm_and_si128(_mm_xor_si128(co_shuffle, co_flip_mask), _mm_set1_epi8(7));

            cube.0 = _mm_or_si128(cp, co);
        }
    }
}
