extern crate core;

use std::primitive;
use std::str::FromStr;
use std::time::Instant;
use clap::Parser;
use itertools::Itertools;
use log::{debug, info, LevelFilter};
use simple_logger::SimpleLogger;
use cubelib::algs::{Algorithm, Solution};
use cubelib::cube::{ApplyAlgorithm, Move, NewSolved, Transformation, Turnable};
use cubelib::cubie::{CubieCube, EdgeCubieCube};
use cubelib::df_search::{NissType, SearchOptions};
use cubelib::coords::dr::DRUDEOFBCoord;
use cubelib::coords::eo::EOCoordFB;
use cubelib::coords::fr::{FRUDNoSliceCoord, FRUDWithSliceCoord};
use cubelib::coords::htr::{ImpureHTRDRUDCoord, PureHTRDRUDCoord};
use cubelib::{lookup_table, stream};
use cubelib::coords::finish::FRFinishCoord;
use cubelib::steps::{dr, eo, finish, fr, htr, step};
use cubelib::steps::finish::fr_finish_leave_slice_any;

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
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    info!("Generating FR pruning table...");
    let frud_table = lookup_table::generate(&fr::FR_UD_MOVESET, &|c: &CubieCube| {
        FRUDWithSliceCoord::from(c)
    });
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();

    info!("Generating FR finish pruning table...");
    let fr_finish_table = lookup_table::generate(&finish::FR_FINISH_MOVESET, &|c: &CubieCube| {
        FRFinishCoord::from(c)
    });
    debug!("Took {}ms", time.elapsed().as_millis());
    let time = Instant::now();


    let scramble = Algorithm::from_str(cli.scramble.as_str()).expect("Invalid scramble {}");

    let mut cube = CubieCube::new_solved();
    cube.apply_alg(&scramble);

    //We want EO, DR and HTR on any axis
    let eo_step = eo::eo_any::<EdgeCubieCube>(&eofb_table);
    let dr_step = dr::dr_any(&drud_eofb_table);
    let htr_step = htr::htr_any(&htr_drud_table);
    let fr_step = fr::fr_any(&frud_table);
    let finish_step = finish::fr_finish_any(&fr_finish_table);

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
    );
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
            NissType::AtStart
        } else {
            NissType::None
        }, step_limit),
        cube.clone(),
    );
    let solutions = step::next_step(
        solutions,
        &finish_step,
        SearchOptions::new(0, 10, NissType::None, step_limit),
        cube.clone(),
    );

    let solutions = solutions
        .skip_while(|alg| alg.len() < cli.min)
        .take_while(|alg| cli.max.map(|max| alg.len() <= max).unwrap_or(true));


    // For FR the direction of the last move always matters so we can't filter if we're doing FR
    // if !cli.all_solutions {
    //     solutions = Box::new(solutions
    //         .filter(|alg| eo::filter_eo_last_moves_pure(&alg.clone().into())));
    // }

    //We already generate a mostly duplicate iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_solutions(solutions);

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions);

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
