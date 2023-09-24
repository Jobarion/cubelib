extern crate core;

use std::arch::x86_64::{__m128i, _mm_set_epi64x};
use std::collections::{HashMap, HashSet};
use std::primitive;
use std::str::FromStr;
use std::time::Instant;

use clap::command;
use clap::Parser;
use log::{debug, info, log, LevelFilter, Log};
use simple_logger::SimpleLogger;

use cubelib::algs::{Algorithm, Solution};
use cubelib::avx2_coord::avx2_coord;
use cubelib::coord::{
    CPCoord, CPOrbitTwistCoord, CPOrbitUnsortedCoord, Coord, DRUDEOFBCoord, EOCoordFB,
    FBSliceUnsortedCoord, HTRDRUDCoord, ImpureHTRDRUDCoord, ParityCoord,
};
use cubelib::cube::{ApplyAlgorithm, Axis, Move, NewSolved, Turnable};
use cubelib::cubie::{CornerCubieCube, CubieCube, EdgeCubieCube};
use cubelib::df_search::{dfs_iter, NissType, SearchOptions};
use cubelib::htr::{HTR_DR_UD_MOVESET, HTR_DR_UD_STATE_CHANGE_MOVES, HTR_MOVES};
use cubelib::step::Step;
use cubelib::{algs, dr, eo, htr, lookup_table, step};

use crate::cli::Cli;

mod cli;

fn main() {
    let cli = Cli::parse();
    SimpleLogger::new()
        .with_level(if cli.verbose {
            LevelFilter::Debug
        } else if cli.quiet {
            LevelFilter::Error
        } else {
            LevelFilter::Info
        })
        .init()
        .unwrap();

    let time = Instant::now();

    info!("Generating EO pruning table...");
    let eofb_table =
        lookup_table::generate(&eo::EO_FB_MOVESET, &|c: &EdgeCubieCube| EOCoordFB::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    info!("Generating DR pruning table...");
    let drud_eofb_table = lookup_table::generate(&dr::DR_UD_EO_FB_MOVESET, &|c: &CubieCube| {
        DRUDEOFBCoord::from(c)
    });
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    info!("Generating HTR pruning table...");
    let htr_drud_table = lookup_table::generate(&htr::HTR_DR_UD_MOVESET, &|c: &CubieCube| {
        ImpureHTRDRUDCoord::from(c)
    });
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    let mut cube = CubieCube::new_solved();

    let scramble = Algorithm::from_str(cli.scramble.as_str()).expect("Invalid scramble {}");
    cube.apply_alg(&scramble);

    // let pre_moves = Algorithm::from_str("D R' U R2 B' U2 L U' R' B2 L D2 L' D").unwrap();
    // cube.apply_alg(&pre_moves);

    //We want EO, DR and HTR on any axis
    let eo_step = eo::eo_any::<EdgeCubieCube>(&eofb_table);
    let dr_step = dr::dr_any(&drud_eofb_table);
    let htr_step = htr::htr_any(&htr_drud_table);

    let solutions = step::first_step(
        &eo_step,
        SearchOptions::new(0, 5, NissType::During),
        cube.edges.clone(),
    );
    let solutions = step::next_step(
        solutions,
        &dr_step,
        SearchOptions::new(0, 14, NissType::AtStart),
        cube.clone(),
    );
    let solutions = step::next_step(
        solutions,
        &htr_step,
        SearchOptions::new(0, 20, NissType::During),
        cube.clone(),
    );

    let solutions = solutions
        .filter(|alg| eo::filter_eo_last_moves_pure(&alg.clone().into()))
        // .take(20)
        ;

    info!("Generating solutions\n");

    //The iterator is always sorted, so this just prints the 20 shortest solution
    for solution in solutions {
        println!("{}", solution);
    }

    debug!("Took {}ms", time.elapsed().as_millis());
}
