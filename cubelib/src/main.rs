use std::str::FromStr;

use log::Level;

use cubelib::algs::Algorithm;
use cubelib::cube::{Cube333, Symmetry, Turn333};
use cubelib::cube::turn::TurnableMut;
use cubelib::solver::lookup_table;
use cubelib::solver::move_table::MoveTable;
use cubelib::solver_new::htr::HTR_TABLES;
use cubelib::steps::coord::Coord;
use cubelib::steps::dr::coords::DRUDEOFBCoord;
use cubelib::steps::finish::coords::{CPCoord, DR_FINISH_SIZE, DRFinishCoord, DRFinishNonSliceEP, DRFinishSliceCoord};
use cubelib::steps::finish::finish_config::DR_UD_FINISH_MOVESET;
use cubelib::steps::htr::coords::HTRDRUDCoord;

fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let symmetries = vec![
        Symmetry::U0, Symmetry::UM0,
        Symmetry::U1, Symmetry::UM1,
        Symmetry::U2, Symmetry::UM2,
        Symmetry::U3, Symmetry::UM3,
        Symmetry::D0, Symmetry::DM0,
        Symmetry::D1, Symmetry::DM1,
        Symmetry::D2, Symmetry::DM2,
        Symmetry::D3, Symmetry::DM3,
    ];

    let non_slice_ep_table = MoveTable::<40320, DRFinishNonSliceEP>::generate_with_symmetries(&DR_UD_FINISH_MOVESET, &symmetries);
    let mut cube = Cube333::default();
    let coord = DRFinishNonSliceEP::from(&cube.edges);
    let c1 = non_slice_ep_table.get(coord, Turn333::Di);
    let c2 = non_slice_ep_table.get(c1, Turn333::D);

    println!("{coord:?} - D' -> {c1:?} - D -> {c2:?}");
    return;



    // let table = lookup_table::generate(&DR_UD_FINISH_MOVESET,
    //                        &|c: &Cube333|DRFinishCoord::min_with_symmetries(c, &symmetries),
    //                        &||lookup_table::SymTable::<{DR_FINISH_SIZE}, DRFinishCoord>::new(),
    //                        &|t, c|t.get(c),
    //                        &|t, c, v|t.set(c, v)
    // );

    lookup_table::generate_dr_finish_table();

    // println!("{:?}", CPCoord::from(&cube.corners));
    // println!("{:?}", CPCoord::min_with_symmetries(&cube.corners, &symmetries));
}