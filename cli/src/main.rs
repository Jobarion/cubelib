use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use cubelib::algs::Algorithm;
use cubelib::defs::StepKind;
use cubelib::puzzles::c333::{Cube333, Turn333};
use cubelib::puzzles::c333::steps::eo;
use cubelib::puzzles::c333::steps::solver::{build_steps, gen_tables};
use cubelib::puzzles::puzzle::ApplyAlgorithm;
use cubelib::solution::Solution;
use cubelib::stream;
use cubelib::tables::PruningTables;
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;

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

    let scramble = Algorithm::from_str(cli.scramble.as_str()).expect("Invalid scramble {}");
    let mut cube = Cube333::default();
    cube.apply_alg(&scramble);

    let steps = cli.parse_step_configs();
    let mut tables = PruningTables::new();

    let steps = if let Err(e) = steps {
        error!("Unable to parse steps config. {e}");
        return;
    } else if let Ok(val) = steps {
        gen_tables(&val, &mut tables);
        build_steps(val, &tables)
    } else {
        unreachable!()
    };

    let steps = if let Err(e) = steps {
        error!("{e}");
        return;
    } else if let Ok(val) = steps {
        val
    } else {
        unreachable!()
    };


    let solutions = cubelib::solver::solve_steps(cube, &steps);

    info!("Generating solutions\n");
    let time = Instant::now();

    let mut solutions: Box<dyn Iterator<Item = Solution<Turn333>>> = Box::new(solutions
        .skip_while(|alg| alg.len() < cli.min)
        .take_while(|alg| cli.max.map_or(true, |max| alg.len() <= max)));


    // For e.g. FR the direction of the last move always matters so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.kind() != StepKind::FR && s.kind() != StepKind::FRLS && s.kind() != StepKind::FIN).unwrap_or(true);
    if !cli.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::eo_config::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    //We already generate a mostly duplicate iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_algorithms(solutions);

    let mut solutions: Box<dyn Iterator<Item = Solution<Turn333>>> = Box::new(solutions);

    if cli.max.is_none() || cli.solution_count.is_some() {
        solutions = Box::new(solutions
            .take(cli.solution_count.unwrap_or(1)))
    }

    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions {
        if cli.compact_solutions {
            if cli.plain_solution {
                println!("{}", Into::<Algorithm<Turn333>>::into(solution));
            } else {
                let alg = Into::<Algorithm<Turn333>>::into(solution);
                println!("{alg} ({})", alg.len());
            }
        } else {
            println!("{}", solution);
        }
    }

    debug!("Took {}ms", time.elapsed().as_millis());
}
