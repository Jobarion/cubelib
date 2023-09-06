extern crate core;

use std::cmp::{max, min};
use std::collections::HashSet;
use std::primitive;
use std::str::FromStr;
use std::time::Instant;
use crate::algs::Algorithm;
use crate::coord::{COCoordUD, Coord, CPCoord, EOCoord, EOCoordSingle, EPCoord};
use crate::cube::{Cube, Face, Move, Turn, Turnable};
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search::{ALL_MOVES, MoveSkipTracker};
use crate::eo::EOCount;
use crate::lookup_table::Table;
use crate::moveset::TransitionTable;
use crate::stream::gen_moves;
// use crate::cubie::CubieCube;

mod facelet;
mod cube;
mod cubie;
mod eo;
mod algs;
mod df_search;
mod dr;
mod alignment;
mod coord;
mod lookup_table;
mod stream;
mod co;
mod rzp;
mod moveset;

fn main() {

    let eoud_table = lookup_table::generate(&|c: &CubieCube| EOCoord::from(&c.edges).0);

    let eoud_drfb_table = lookup_table::generate(&|c: &CubieCube| {
        let eo_data = EOCoord::from(&c.edges);
        let co_data = COCoordUD::from(&c.corners);
        EOCoord::from(&c.edges).0
    });


    let time = Instant::now();

    let scramble = Algorithm { normal_moves: algs::parse_algorithm("D"), inverse_moves: vec![] };
    let mut cube = cubie::CubieCube::new_solved();
    cube.apply(&scramble);


    // let a: Vec<Algorithm> = eo::eo_ud_state_iter::<CubieCube>(&cube)
    //     .take_while(|eo|eo.len() <= 5)
    //     .collect();
    // println!("Count 1: {}", a.len());


    let b: Vec<Algorithm> = eo::eo_ud_iter_table_heuristic(&cube.edges, &eoud_table)
        .take_while(|eo|eo.len() <= 3)
        .filter(|alg| eo::filter_eo_last_moves_pure(&alg))
        .collect();
    println!("Count 2: {}", b.len());

    for x in b.iter() {
        // let mut cube = cube.clone();
        // cube.apply(&x);
        println!("{x}");
    }
    //
    // // println!("{}", "\n".repeat(5));
    //
    // // println!("???");
    // // println!("{cube}");
    //
    // let mut a_set: HashSet<Algorithm> = HashSet::new();
    // for alg_a in a.into_iter() {
    //     a_set.insert(alg_a);
    // }
    // let mut b_set: HashSet<Algorithm> = HashSet::new();
    // for alg_b in b.into_iter() {
    //     b_set.insert(alg_b);
    // }
    // for a in a_set.iter() {
    //     if !b_set.contains(&a) {
    //         let mut c = cube.clone();
    //         c.apply(&a);
    //         println!("Mismatch not in b: {a} {:?}", c.count_bad_edges());
    //     }
    // }
    // for b in b_set.iter() {
    //     if !a_set.contains(&b) {
    //         println!("Mismatch not in a: {b}");
    //     }
    // }

    // let edge_cube = cube.edges;
    // let count = eo::eo_state_iter_table::<0, 0, 0, EdgeCubieCube>(&edge_cube, &eo_table)
    //     .take_while(|eo|eo.len() <= 5)
    //     .count();
    // println!("Count table {}", count);


    // let rzp = eo::any_eo_iter(&cube)
    //     .take_while(|eo|eo.len() <= 5)
    //     .flat_map(|eo_alg| {
    //         let mut cube = cube.clone();
    //         cube.apply(&eo_alg);
    //         let rzp_length = min(6 - eo_alg.len(), 2);
    //
    //         let eo = cube.edges.count_bad_edges();
    //         let eo = (eo.0 == 0, eo.1 == 0, eo.2 == 0);
    //
    //         let ud = eo.0.then_some(gen_moves::<CubieCube, 14>(UD_EO_MOVES, rzp_length)(eo_alg.clone())).into_iter().flatten();
    //         let fb = eo.1.then_some(gen_moves::<CubieCube, 14>(FB_EO_MOVES, rzp_length)(eo_alg.clone())).into_iter().flatten();
    //         let rl = eo.2.then_some(gen_moves::<CubieCube, 14>(RL_EO_MOVES, rzp_length)(eo_alg.clone())).into_iter().flatten();
    //
    //         ud.chain(fb).chain(rl)
    //     })
    //     .count();
    // println!("RZP count {}", rzp);

    // let a = lookup_table::generate(&|c: &CubieCube| EOCoord::from(c.edges));
    // println!("{:?}", a.get(EOCoord::from(cube.edges)));

    // println!("{}", CubieCube::CORNER_COLORS[5][1]);


    // println!("{:?}", EPCoord::from(cube.edges));
    // cube.turn(Move::from_str("D").unwrap());
    // println!("{}", cube);
    // println!("{:?}", EPCoord::from(cube.edges));
    let moves = algs::parse_algorithm("U R F B D B L F' U' B' R' D' L' ".repeat(10).as_str());
    let mut sum = 0_u32;


    println!("{}", sum);

    println!("Took {}ms", time.elapsed().as_millis());
}
