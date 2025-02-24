use std::str::FromStr;
use cubelib::algs::Algorithm;
use cubelib::puzzles::c333::Cube333;
use cubelib::puzzles::puzzle::ApplyAlgorithm;

pub fn main() {
    let mut cube = Cube333::default();
    let tperm = Algorithm::from_str("R U R' U' R' F R2 U' R' U' R U R' F'").unwrap();
    for _ in 0..100_000_000 {
        cube.apply_alg(&tperm);
    }
    println!("{cube}");
}