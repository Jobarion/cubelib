use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use cubelib::cube::turn::InvertibleMut;

use cubelib::solver::df_search::CancelToken;
use cubelib::steps::{solver};
use cubelib::steps::tables::PruningTables333;
use log::{error, info};
use simple_logger::SimpleLogger;
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::solver::solution::Solution;

use crate::cli::{Cli, SolutionFormat, SolveCommand, InvertCommand};

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
    };

    find_and_print_solutions(cube, cmd);
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

    find_and_print_solutions(cube, cmd);
}

fn find_and_print_solutions(cube: Cube333, cmd: SolveCommand) {
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
        }
        Err(e) => {
            error!("Unable to parse steps config. {e}");
            return;
        }
    };

    let cancel_token = CancelToken::default();
    let time = Instant::now();

    let mut solutions = cubelib::solver::solve_steps(cube, &steps, &cancel_token);
    if !cmd.all_solutions {
        solutions = solutions.into_iter().filter(Solution::is_canonical).collect();
    }
    let limit = cmd.solution_count.unwrap_or(usize::MAX);
    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions.into_iter().take(limit) {
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


