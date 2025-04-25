use std::str::FromStr;

use log::Level;

use cubelib::algs::Algorithm;
use cubelib::cube::{Cube333, Symmetry};
use cubelib::solver::lookup_table;
use cubelib::steps::coord::Coord;
use cubelib::steps::finish::coords::{CPCoord, DR_FINISH_SIZE, DRFinishCoord};
use cubelib::steps::finish::finish_config::DR_UD_FINISH_MOVESET;

fn main() {
    let mut cube: Cube333 = Algorithm::from_str("R").unwrap().into();
    // println!("{}", C)

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

    simple_logger::init_with_level(Level::Debug).unwrap();
    let table = lookup_table::generate(&DR_UD_FINISH_MOVESET,
                           &|c: &Cube333|DRFinishCoord::min_with_symmetries(c, &symmetries),
                           &||lookup_table::SymTable::<{DR_FINISH_SIZE}, DRFinishCoord>::new(),
                           &|t, c|t.get(c),
                           &|t, c, v|t.set(c, v)
    );

    println!("{:?}", CPCoord::from(&cube.corners));
    println!("{:?}", CPCoord::min_with_symmetries(&cube.corners, &symmetries));
}