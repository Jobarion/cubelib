use std::str::FromStr;

use cubelib::algs::Algorithm;
use cubelib::puzzles::c333::Cube333;
use cubelib::puzzles::c333::steps::finish::coords::HTRFinishCoord;
use cubelib::puzzles::puzzle::ApplyAlgorithm;

pub fn main() {
    let mut cube = Cube333::default();
    let alg = Algorithm::from_str("U2 F2 D2 L2 R2 B2").unwrap();
    cube.apply_alg(&alg);
    println!("{:?}", HTRFinishCoord::from(&cube));
}