extern crate core;

use std::collections::HashMap;
use home::home_dir;
use std::fs;
use std::io::ErrorKind;
use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use cubelib::algs::Algorithm;
use cubelib::cube::*;
use cubelib::cube::turn::InvertibleMut;
use cubelib::defs::{NissSwitchType, StepKind};
use cubelib::solver::df_search::CancelToken;
use cubelib::solver::solution::Solution;
use cubelib::solver::stream;
use cubelib::solver_new::util_steps::{FilterDup, FilterLastMoveNotPrime};
use cubelib::steps::{eo, solver};
use cubelib::steps::step::StepConfig;
use cubelib::steps::tables::PruningTables333;
use log::{error, info, log};
use regex::Regex;
use simple_logger::SimpleLogger;
use crate::cli::{Cli, InvertCommand, LogLevel, SolutionFormat, SolveCommand, SolverBackend};
use crate::config::{SolverConfig, CubelibConfig};

mod cli;
mod steps;
mod config;

fn main() {
    let cli: Cli = Cli::parse();

    let mut messages = vec![];

    let mut dir = home_dir().unwrap();
    dir.push(".cubelib");
    dir.push("config.toml");

    messages.push((LogLevel::Info, format!("Reading config from {dir:?}")));
    let mut config: CubelibConfig = match fs::read_to_string(dir.to_str().expect("Valid path")) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            messages.push((LogLevel::Error, "Config file contains errors, using defaults.".to_string()));
            messages.push((LogLevel::Error, e.to_string()));
            CubelibConfig::default()
        }),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                messages.push((LogLevel::Info, "No config file found, using defaults.".to_string()));
            } else {
                messages.push((LogLevel::Error, "Error reading config file, using defaults.".to_string()));
                messages.push((LogLevel::Error, e.to_string()));
            }
            CubelibConfig::default()
        }
    };

    if let Some(log) = cli.log {
        config.log = log;
    }
    SimpleLogger::new()
        .with_level(config.log.to_level_filter())
        .init()
        .unwrap();

    for (level, message) in messages {
        if let Some(level) = level.to_level_filter().to_level() {
            log!(level, "{message}");
        }
    }

    match {
        cli.command
    } {
        cli::Commands::Solve(cmd) => solve(cmd, config.solver_config),
        cli::Commands::Invert(cmd) => invert(cmd),
        cli::Commands::Scramble => scramble(),
    }
}

fn scramble() {
    let cube = Cube333::random(&mut rand::rng());

    let mut solver_config = SolverConfig::default();
    solver_config.format = SolutionFormat::Plain;
    solver_config.solution_count = Some(1);
    solver_config.quality = 1000;
    solver_config.steps = "EO[max=7;niss=never] > DR[niss=never] > HTR[niss=never] > FIN[niss=never]".to_string();

    find_and_print_solutions_iter_stream(cube, solver_config);
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

fn solve(cmd: SolveCommand, mut config: SolverConfig) {
    let scramble = read_scramble(&cmd.scramble);
    let cube = scramble.into();

    config.merge_cli_parameters(cmd);

    match config.backend {
        SolverBackend::IterStream => find_and_print_solutions_iter_stream(cube, config),
        SolverBackend::MultiPathChannel => find_and_print_solutions_multi_path_channel(cube, config),
    }
}

fn find_and_print_solutions_iter_stream(cube: Cube333, config: SolverConfig) {
    let mut tables = PruningTables333::new();

    let steps = match parse_step_configs_iter_stream(&config) {
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
        .skip_while(|alg| alg.len() < config.min)
        .take_while(|alg| config.max.map_or(true, |max| alg.len() <= max)));

    // For e.g. FR the direction of the last move always matters, so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.kind() != StepKind::FR && s.kind() != StepKind::FIN).unwrap_or(true);
    if !config.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::eo_config::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    //We already generate a mostly duplicate free iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_algorithms(solutions);

    let mut solutions: Box<dyn Iterator<Item=Solution>> = Box::new(solutions);

    if config.max.is_none() || config.solution_count.is_some() {
        solutions = Box::new(solutions
            .take(config.solution_count.unwrap_or(1)))
    }

    //The iterator is always sorted, so this just prints the shortest solutions
    for solution in solutions {
        match config.format {
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

fn parse_step_configs_iter_stream(conf: &SolverConfig) -> Result<Vec<StepConfig>, String> {
    let step_name_regex = Regex::new("[A-Za-z0-9_-]").unwrap();
    let default_niss_type = None;
    conf.steps.split(">")
        .map(|step| step.trim())
        .map(|step| {
            let param_start = step.find("[");
            if param_start.is_none() {
                if !step_name_regex.is_match(step) {
                    return Err(format!("Invalid step name {}", step));
                }
                return Ok(StepConfig {
                    kind: StepKind::from_str(step).unwrap(),
                    substeps: None,
                    min: None,
                    max: None,
                    absolute_min: None,
                    absolute_max: None,
                    niss: default_niss_type,
                    step_limit: None,
                    quality: conf.quality,
                    params: HashMap::new()
                });
            } else {
                if !step.ends_with("]") {
                    return Err(format!("Expected step parameters to end with ] {}", step));
                }
                let param_start = param_start.unwrap();
                let name = &step[0..param_start];
                let mut step_prototype = StepConfig {
                    kind: StepKind::from_str(name).unwrap(),
                    substeps: None,
                    min: None,
                    max: None,
                    absolute_min: None,
                    absolute_max: None,
                    niss: default_niss_type,
                    step_limit: None,
                    quality: conf.quality,
                    params: HashMap::new()
                };
                let params: Vec<&str> = (&step[(param_start + 1)..(step.len() - 1)]).split(";").collect();
                for param in params {
                    if param.contains("=") {
                        let parts: Vec<&str> = param.split("=").collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid param format {}", param));
                        }
                        match parts[0] {
                            "limit" => step_prototype.step_limit = Some(usize::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for count. '{x}'", parts[1]))?),
                            key @ "min" | key @ "min-rel" => step_prototype.min = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for {key}. '{x}'", parts[1]))?),
                            key @ "max" | key @ "max-rel" => step_prototype.max = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for {key}. '{x}'", parts[1]))?),
                            "min-abs" => step_prototype.absolute_min = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for min-abs. '{x}'", parts[1]))?),
                            "max-abs" => step_prototype.absolute_max = Some(u8::from_str(parts[1]).map_err(|x| format!("Unable to parse value '{}' for max-abs. '{x}'", parts[1]))?),
                            "niss" => step_prototype.niss = Some(match parts[1] {
                                "always" | "true" => NissSwitchType::Always,
                                "before" => NissSwitchType::Before,
                                "none" | "never" | "false" => NissSwitchType::Never,
                                x => Err(format!("Invalid NISS type {x}. Expected one of 'always', 'before', 'none'"))?
                            }),
                            key => {
                                step_prototype.params.insert(key.to_string(), parts[1].to_string());
                            }
                        }
                    } else {
                        step_prototype.substeps = Some(step_prototype.substeps.map_or(vec![param.to_string()], |mut v|{
                            v.push(param.to_string());
                            v
                        }));
                    }
                }
                Ok(step_prototype)
            }
        })
        .collect()
}

fn find_and_print_solutions_multi_path_channel(cube: Cube333, config: SolverConfig) {
    let cube_state = cube.get_cube_state();
    let (mut steps, last_step) = match steps::parse_steps(&config.steps, &config.get_merged_overrides(), cube_state) {
        Ok(x) => x,
        Err(e) => {
            error!("Unable to parse steps config. {e}");
            return;
        }
    };

    info!("Generating solutions\n");
    let time = Instant::now();

    let mut predicates = vec![];

    let last_qt_diretion_relevant = match last_step {
        StepKind::EO | StepKind::RZP | StepKind::DR | StepKind::HTR => false,
        _ => true
    };

    if !config.all_solutions && !last_qt_diretion_relevant  {
        predicates.push(FilterLastMoveNotPrime::new());
    }
    predicates.push(FilterDup::new());
    steps.with_predicates(predicates);

    if config.quality > 0 {
        steps.apply_step_limit(config.quality);
    }

    let mut worker = steps.into_worker(cube);

    let mut count = 0;
    let max_length = config.solution_count.or(if config.max.is_some() {
        None
    } else {
        Some(1)
    });
    while max_length.is_none() || max_length.unwrap() > count {
        match worker.next() {
            Some(solution) => {
                if solution.len() < config.min {
                    continue;
                }
                if config.max.map(|max|solution.len() > max).unwrap_or(false) {
                    break
                }
                match config.format {
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
            None => break
        }
    }

    info!("Took {}ms", time.elapsed().as_millis());
}

