use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use cubelib::cube::turn::InvertibleMut;
use cubelib::defs::StepKind;

use cubelib::solver::stream;
use cubelib::solver::df_search::CancelToken;
use cubelib::solver::solution::Solution;
use cubelib::steps::{eo, solver};
use cubelib::steps::tables::PruningTables333;
use log::{error, info};
use simple_logger::SimpleLogger;
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::solver_new::util_steps::{FilterDup, FilterLastMoveNotPrime};
use crate::cli::{Cli, SolutionFormat, SolveCommand, InvertCommand, SolverBackend};

mod cli;

fn main() {
    let cli: Cli = Cli::parse();
    SimpleLogger::new()
        .with_level(cli.log.to_level_filter())
        .init()
        .unwrap();

    match {
        cli.command
    } {
        cli::Commands::Solve(cmd) => solve(cmd),
        cli::Commands::Invert(cmd) => invert(cmd),
        cli::Commands::Scramble => scramble(),
    }
}

fn scramble() {
    let cube = Cube333::random(&mut rand::thread_rng());

    let cmd = SolveCommand {
        format: SolutionFormat::Plain,
        all_solutions: false,
        min: 0,
        max: None,
        niss: false,
        solution_count: Some(1),
        quality: 100,
        steps: "EO[max=6] > DR > HTR > Finish".to_string(),
        scramble: "".to_string(),
        backend: SolverBackend::IterStream,
    };

    find_and_print_solutions_iter_stream(cube, cmd);

}

fn read_scramble(input: &String) -> Algorithm {
    match input.as_str() {
        "-" => {
            // Read from stdin
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read from stdin");
            Algorithm::from_str(input.trim()).expect("Invalid scramble")
        }
        s => Algorithm::from_str(s).expect("Invalid scramble {}")
    }
}

fn invert(cmd: InvertCommand) {
    let mut scramble = read_scramble(&cmd.scramble);
    scramble.invert();
    println!("{}", scramble);
}

fn solve(cmd: SolveCommand) {
    let scramble = read_scramble(&cmd.scramble);
    let mut cube = Cube333::default();
    cube.apply_alg(&scramble);

    match cmd.backend {
        SolverBackend::IterStream => find_and_print_solutions_iter_stream(cube, cmd),
        SolverBackend::MultiPathChannel => find_and_print_solutions_multi_path_channel(cube, cmd),
    }

}

fn find_and_print_solutions_iter_stream(cube: Cube333, cmd: SolveCommand) {
    let mut tables = PruningTables333::new();

    let steps = match cmd.parse_step_configs() {
        Ok(step_configs) => {
              solver::gen_tables(&step_configs, &mut tables);
                match solver::build_steps(step_configs, &tables) {
                    Ok(steps) => steps,
                    Err(e) => {
                        error!("{e}");
                        return;
                    }
                }
        },
        Err(e) => {
            error!("Unable to parse steps config. {e}");
            return;
        }
    };

    let cancel_token = CancelToken::default();
    let solutions = cubelib::solver::solve_steps(cube, &steps, &cancel_token);

    info!("Generating solutions\n");
    let time = Instant::now();

    let mut solutions: Box<dyn Iterator<Item=Solution>> = Box::new(solutions
        .skip_while(|alg| alg.len() < cmd.min)
        .take_while(|alg| cmd.max.map_or(true, |max| alg.len() <= max)));

    // For e.g. FR the direction of the last move always matters, so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.kind() != StepKind::FR && s.kind() != StepKind::FIN).unwrap_or(true);
    if !cmd.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::eo_config::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    //We already generate a mostly duplicate free iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_algorithms(solutions);

    let mut solutions: Box<dyn Iterator<Item=Solution>> = Box::new(solutions);

    if cmd.max.is_none() || cmd.solution_count.is_some() {
        solutions = Box::new(solutions
            .take(cmd.solution_count.unwrap_or(1)))
    }

    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions {
        match cmd.format {
            SolutionFormat::Plain =>
                println!("{}", Into::<Algorithm>::into(solution)),
            SolutionFormat::Compact => {
                let alg = Into::<Algorithm>::into(solution);
                println!("{alg} ({})", alg.len());
            }
            SolutionFormat::Detailed =>
                println!("{}", solution)
        }
    }

    info!("Took {}ms", time.elapsed().as_millis());
}

fn find_and_print_solutions_multi_path_channel(cube: Cube333, cmd: SolveCommand) {
    let (mut steps, last_step) = match cmd.parse_step_configs() {
        Ok(step_configs) => {
            let last = step_configs.last().unwrap().clone().kind;
            match cubelib::solver_new::build_steps(step_configs) {
                Ok(steps) => (steps, last),
                Err(e) => {
                    error!("{e}");
                    return;
                }
            }
        },
        Err(e) => {
            error!("Unable to parse steps config. {e}");
            return;
        }
    };

    info!("Generating solutions\n");
    let time = Instant::now();

    let mut predicates = vec![FilterDup::new()];

    let last_qt_diretion_relevant = match last_step {
        StepKind::EO | StepKind::RZP | StepKind::DR | StepKind::HTR => false,
        _ => true
    };

    if !cmd.all_solutions && !last_qt_diretion_relevant  {
        predicates.push(FilterLastMoveNotPrime::new());
    }

    if cmd.quality > 0 {
        steps.apply_step_limit(cmd.quality);
    }

    let (mut worker, rec) = cubelib::solver_new::create_worker_with_predicates(cube, steps, predicates);
    worker.start();

    let mut count = 0;
    while cmd.solution_count.is_none() || cmd.solution_count.unwrap() > count {
        match rec.recv() {
            Ok(solution) => {
                if solution.len() < cmd.min {
                    continue;
                }
                if cmd.max.map(|max|solution.len() > max).unwrap_or(false) {
                    break
                }
                match cmd.format {
                    SolutionFormat::Plain =>
                        println!("{}", Into::<Algorithm>::into(solution)),
                    SolutionFormat::Compact => {
                        let alg = Into::<Algorithm>::into(solution);
                        println!("{alg} ({})", alg.len());
                    }
                    SolutionFormat::Detailed =>
                        println!("{}", solution)
                }
                count += 1;
            },
            Err(_) => break
        }
    }

    info!("Took {}ms", time.elapsed().as_millis());
    drop(rec);
    worker.stop();
}

