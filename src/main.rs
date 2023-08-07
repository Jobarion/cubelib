use std::time::Instant;
use crate::cube::{Cube, Face, Turn};
// use crate::cubie::CubieCube;

mod facelet;
mod cube;
mod cubie;

fn main() {
    let mut cube = cubie::CubieCube::new_solved();

    let time = Instant::now();
    for n in 0..100_000_000 {
        cube.turn(Face::Right, Turn::Clockwise);
        cube.turn(Face::Up, Turn::Clockwise);
        cube.turn(Face::Right, Turn::CounterClockwise);
        cube.turn(Face::Up, Turn::CounterClockwise);
        cube.turn(Face::Right, Turn::CounterClockwise);
        cube.turn(Face::Front, Turn::Clockwise);
        cube.turn(Face::Right, Turn::Half);
        cube.turn(Face::Up, Turn::CounterClockwise);
        cube.turn(Face::Right, Turn::CounterClockwise);
        cube.turn(Face::Up, Turn::CounterClockwise);
        cube.turn(Face::Right, Turn::Clockwise);
        cube.turn(Face::Up, Turn::Clockwise);
        cube.turn(Face::Right, Turn::CounterClockwise);
        cube.turn(Face::Front, Turn::CounterClockwise);
    }
    println!("Took {}ms", time.elapsed().as_millis());
}
