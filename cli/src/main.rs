use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use cubelib::defs::StepKind;

use cubelib::solver::stream;
use cubelib::solver::df_search::CancelToken;
use cubelib::solver::solution::Solution;
use cubelib::steps::{eo, solver};
use cubelib::steps::tables::PruningTables333;
use log::{error, info};
use simple_logger::SimpleLogger;
use cubelib::cube::turn::ApplyAlgorithm;

use crate::cli::{Cli, SolutionFormat};

mod cli;

fn main() {
    let cli: Cli = Cli::parse();
    SimpleLogger::new()
        .with_level(cli.log.to_level_filter())
        .init()
        .unwrap();

    let scramble = Algorithm::from_str(cli.scramble.as_str()).expect("Invalid scramble {}");
    let mut cube = Cube333::default();
    cube.apply_alg(&scramble);

    let steps = cli.parse_step_configs();
    let mut tables = PruningTables333::new();

    let steps = if let Err(e) = steps {
        error!("Unable to parse steps config. {e}");
        return;
    } else if let Ok(val) = steps {
        solver::gen_tables(&val, &mut tables);
        solver::build_steps(val, &tables)
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


    let cancel_token = CancelToken::default();
    let solutions = cubelib::solver::solve_steps(cube, &steps, &cancel_token);

    info!("Generating solutions\n");
    let time = Instant::now();

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions
        .skip_while(|alg| alg.len() < cli.min)
        .take_while(|alg| cli.max.map_or(true, |max| alg.len() <= max)));

    // For e.g. FR the direction of the last move always matters, so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.kind() != StepKind::FR && s.kind() != StepKind::FIN).unwrap_or(true);
    if !cli.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::eo_config::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    //We already generate a mostly duplicate free iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_algorithms(solutions);

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions);

    if cli.max.is_none() || cli.solution_count.is_some() {
        solutions = Box::new(solutions
            .take(cli.solution_count.unwrap_or(1)))
    }

    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions {
        match cli.format {
            SolutionFormat::Plain =>
                println!("{}", Into::<Algorithm>::into(solution)),
            SolutionFormat::Compact => {
                let alg = Into::<Algorithm>::into(solution);
                println!("{alg} ({})", alg.len());
            },
            SolutionFormat::Detailed =>
                println!("{}", solution)
        }
    }

    info!("Took {}ms", time.elapsed().as_millis());
}
