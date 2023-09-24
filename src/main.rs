extern crate core;

use std::str::FromStr;
use std::time::Instant;
use clap::Parser;
use log::{debug, info, LevelFilter};
use simple_logger::SimpleLogger;
use cubelib::algs::{Algorithm, Solution};
use cubelib::coord::{DRUDEOFBCoord, EOCoordFB, FBSliceUnsortedCoord, FRUDNoSliceCoord, ImpureHTRDRUDCoord};
use cubelib::cube::{ApplyAlgorithm, Axis, NewSolved, Transformation, Turnable};
use cubelib::cubie::{CubieCube, EdgeCubieCube};
use cubelib::df_search::{NissType, SearchOptions};
use cubelib::{dr, eo, htr, fr, lookup_table, step};
use cubelib::cube::Turn::Clockwise;

use crate::cli::Cli;

mod cli;

fn main() {
    let cli: Cli = Cli::parse();
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

    info!("Generating FR pruning table...");
    let frud_htr_table = lookup_table::generate(&fr::FR_UD_MOVESET, &|c: &CubieCube| {
        FRUDNoSliceCoord::from(c)
    });

    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    let mut cube = CubieCube::new_solved();

    let scramble = Algorithm::from_str(cli.scramble.as_str()).expect("Invalid scramble {}");

    cube.apply_alg(&scramble);


    //We want EO, DR and HTR on any axis
    let eo_step = eo::eo_any::<EdgeCubieCube>(&eofb_table);
    let dr_step = dr::dr_any(&drud_eofb_table);
    let htr_step = htr::htr_any(&htr_drud_table);
    let fr_step = fr::fr_no_slice_any(&frud_htr_table);

    //Default limit of 100 if nothing is set.
    let step_limit = cli.step_limit.or(Some(100).filter(|_|!cli.optimal));
    let solutions = step::first_step(
        &eo_step,
        SearchOptions::new(0, 5, if cli.niss {
            NissType::During
        } else {
            NissType::None
        }, step_limit),
        cube.edges.clone(),
    ).take(100);
    let solutions = step::next_step(
        solutions,
        &dr_step,
        SearchOptions::new(0, 13, if cli.niss {
            NissType::AtStart
        } else {
            NissType::None
        }, step_limit),
        cube.clone(),
    );
    let solutions = step::next_step(
        solutions,
        &htr_step,
        SearchOptions::new(0, 14, if cli.niss {
            NissType::During
        } else {
            NissType::None
        }, step_limit),
        cube.clone(),
    );
    let solutions = step::next_step(
        solutions,
        &fr_step,
        SearchOptions::new(0, 14, if cli.niss {
            NissType::During
        } else {
            NissType::None
        }, step_limit),
        cube.clone(),
    );

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions);

    solutions = Box::new(solutions
        .skip_while(|alg| alg.len() < cli.min));

    if cli.max.is_some() {
        solutions = Box::new(solutions
            .take_while(|alg| alg.len() <= cli.max.unwrap()));
    }

    if !cli.all_solutions {
        solutions = Box::new(solutions
            .filter(|alg| eo::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    if cli.max.is_none() || cli.solution_count.is_some() {
        solutions = Box::new(solutions
            .take(cli.solution_count.unwrap_or(1)))
    }

    info!("Generating solutions\n");

    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions {
        if cli.compact_solutions {
            if cli.plain_solution {
                println!("{}", Into::<Algorithm>::into(solution));
            } else {
                let alg = Into::<Algorithm>::into(solution);
                println!("{alg} ({})", alg.len());
            }
        } else {
            println!("{}", solution);
        }
    }

    debug!("Took {}ms", time.elapsed().as_millis());
}
