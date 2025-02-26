use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::puzzles::c333::Cube333;
use cubelib::puzzles::c333::steps::fr::coords::{FRUDNoSliceCoord, FRUDWithSliceCoord};
use cubelib::puzzles::puzzle::ApplyAlgorithm;

pub fn main() {
    let mut cube = Cube333::default();
    let alg = Algorithm::from_str("U2").unwrap();
    cube.apply_alg(&alg);
    println!("{:?}", FRUDNoSliceCoord::from(&cube));
}