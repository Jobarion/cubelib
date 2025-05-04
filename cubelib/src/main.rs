use std::str::FromStr;
use cubelib::algs::Algorithm;
use cubelib::cube::Cube333;
use cubelib::steps::coord::CPCoord;

pub fn main() {
    let cube: Cube333 = Algorithm::from_str("U2 L2 B2 U2 F2 D B2 U B2 U' F2 U B' F' R2 B F'").unwrap().into();
    println!("{:?}", CPCoord::from(&cube));
}