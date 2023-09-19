extern crate core;

use std::time::Instant;
use cubelib::{algs, dr, eo, lookup_table, step};
use cubelib::algs::Algorithm;
use cubelib::coord::{DRUDEOFBCoord, EOCoordFB};
use cubelib::cube::{ApplyAlgorithm, Axis, NewSolved};
use cubelib::cubie::CubieCube;
use cubelib::df_search::{NissType, SearchOptions};

fn main() {
    let time = Instant::now();

    println!("Generating EO pruning table...");
    let eofb_table = lookup_table::generate(&eo::EO_FB_MOVESET, &|c: &CubieCube| EOCoordFB::from(&c.edges));
    println!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    println!("Generating DR pruning table...");
    let drud_eofb_table = lookup_table::generate(&dr::DR_UD_EO_FB_MOVESET, &|c: &CubieCube| DRUDEOFBCoord::from(c));
    println!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    let mut cube = CubieCube::new_solved();

    let scramble = Algorithm { normal_moves: algs::parse_algorithm("R' U' F U F2 D U2 L2 D R2 U' L2 R U' F2 L' U2 L' F' L2 U2 L F R' U' F"), inverse_moves: vec![] };
    cube.apply_alg(&scramble);


    //We want EO on any axis
    let eo_step = eo::eo_any(&eofb_table);

    //We want DR with any EO axis, but only with CO on the FB or LR axis
    //Only allowing something very specific like eoud-drfb and eolr-drud requires creating the step struct manually.
    //I don't know when this would ever be useful, but it's possible
    let dr_step = dr::dr(&drud_eofb_table, [Axis::UD, Axis::FB, Axis::LR], [Axis::FB, Axis::LR]);
    // let dr_step = dr::dr_any(&drud_eofb_table);

    //We want to find all EOs between 0 and 5 moves. Using NISS is allowed at any point during the solve (so NISS EOs are fine)
    let eo_solutions = step::first_step(&eo_step, SearchOptions::new(0, 5, NissType::During), cube.edges.clone());
    //We want to find all DRs between 0 and 14 moves. This is the length of the entire solution, not just DR. The longest permitted DR on a 5 move EO is therefore 9 moves.
    //Using NISS is only allowed at the start (which means we'll look for a solution that's entirely on the normal or the inverse after applying whatever EO we want to check
    let dr_solutions = step::next_step(eo_solutions, &dr_step, SearchOptions::new(0, 14, NissType::AtStart), cube.clone())
        .filter(|alg| eo::filter_eo_last_moves_pure(alg)); //A DR is a kind of EO in a sense, so we can use the same filter method.


    //The iterator is always sorted, so this just prints the 20 shortest solution
    for a in dr_solutions.take(20) {
        println!("{a}");
    }

    println!("Took {}ms", time.elapsed().as_millis());
}
