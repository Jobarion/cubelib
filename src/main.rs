extern crate core;

use std::arch::x86_64::{__m128i, _mm_set_epi64x};
use std::time::Instant;
use cubelib::{algs, dr, eo, htr, lookup_table, step};
use cubelib::algs::Algorithm;
use cubelib::avx2_coord::avx2_coord;
use cubelib::coord::{CPOrbitTwistParityCoord, CPOrbitUnsortedCoord, DRUDEOFBCoord, EOCoordFB, FBSliceUnsortedCoord, HTRDRUDCoord, ParityCoord};
use cubelib::cube::{ApplyAlgorithm, Axis, Move, NewSolved, Turnable};
use cubelib::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};
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

    println!("Generating HTR pruning table...");
    let htr_drud_table = lookup_table::generate(&htr::HTR_DR_UD_MOVESET, &|c: &CubieCube| HTRDRUDCoord::from(c));
    println!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    let mut cube = CubieCube::new_solved();

    let mut cube = unsafe {
        CubieCube::new(EdgeCubieCube::new(_mm_set_epi64x(146286744, 4638831107911485496)), CornerCubieCube::new(_mm_set_epi64x(0, -9214363874447277920)))
    };

    println!("{cube}");


    let scramble = Algorithm { normal_moves: algs::parse_moves("R' U' F R2 D2 B2 D2 F2 U2 B U2 R2 B' F' R D2 U F' R2 U' F L D R' U' F"), inverse_moves: vec![] };
    cube.apply_alg(&scramble);

    // let pre_moves = Algorithm { normal_moves: algs::parse_moves("L B' L F2 R U R2 D' L U' L2 D'"), inverse_moves: algs::parse_moves("B") };
    // cube.apply_alg(&pre_moves);


    //We want EO, DR and HTR on any axis
    let eo_step = eo::eo_any::<EdgeCubieCube>(&eofb_table);
    let dr_step = dr::dr_any(&drud_eofb_table);
    let htr_step = htr::htr(&htr_drud_table, [Axis::UD]);

    let solutions = step::first_step(&eo_step, SearchOptions::new(0, 5, NissType::During), cube.edges.clone());
    let solutions = step::next_step(solutions, &dr_step, SearchOptions::new(0, 14, NissType::AtStart), cube.clone());
    let solutions = step::next_step(solutions, &htr_step, SearchOptions::new(0, 20, NissType::During), cube.clone());

    let solutions = solutions
        .filter(|alg| eo::filter_eo_last_moves_pure(alg));

    // //The iterator is always sorted, so this just prints the 20 shortest solution
    for a in solutions.take(10) {
        let mut c = cube.clone();
        c.apply_alg(&a);
        println!("{a}");
    }

    println!("Took {}ms", time.elapsed().as_millis());
}
