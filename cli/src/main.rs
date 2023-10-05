mod cli;

use std::str::FromStr;
use std::time::Instant;
use std::vec;

use clap::Parser;
use itertools::Itertools;
use log::{debug, error, info, LevelFilter, warn};
use simple_logger::SimpleLogger;

use cubelib::algs::{Algorithm, Solution};
use cubelib::cube::{ApplyAlgorithm, Axis, Invertible, Move, NewSolved, Transformation, Turnable};
use cubelib::cubie::CubieCube;
use cubelib::steps::{dr, dr_trigger, eo, finish, fr, htr, rzp, step};
use cubelib::steps::step::{DefaultStepOptions, Step, StepConfig, StepKind};
use cubelib::stream;
use cubelib::tables::PruningTables;
use crate::cli::Cli;

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
    let mut cube = CubieCube::new_solved();
    cube.apply_alg(&scramble);

    let steps = cli.parse_step_configs();

    let steps = if let Err(e) = steps {
        error!("Unable to parse steps config. {e}");
        return;
    } else if let Ok(val) = steps {
        let previous = vec![None].into_iter().chain(val.iter().map(|x|Some(x.kind))).collect_vec();
        val.into_iter().zip(previous.into_iter()).collect_vec()
    } else {
        unreachable!()
    };

    let mut tables = PruningTables::new();
    for (conf, pre) in steps.iter() {
        match (pre.clone(), conf.kind.clone()) {
            (_, StepKind::EO) => tables.gen_eo(),
            (_, StepKind::DR) => tables.gen_dr(),
            (_, StepKind::HTR) => tables.gen_htr(),
            (_, StepKind::FR) => tables.gen_fr(),
            (_, StepKind::FRLS) => tables.gen_fr_leave_slice(),
            (Some(StepKind::FR), StepKind::FIN) | (Some(StepKind::FRLS), StepKind::FIN) => tables.gen_fr_finish(),
            _ => ()
        }
    }

    let steps: Result<Vec<(Step<CubieCube>, DefaultStepOptions)>, String> = steps.into_iter()
        .flat_map(|(config, previous)| match (previous, config.kind) {
            (None, StepKind::EO) => vec![eo::from_step_config::<CubieCube>(tables.eo().expect("EO table required"), config)].into_iter(),
            (Some(StepKind::EO), StepKind::RZP)   => vec![rzp::from_step_config::<CubieCube>(config)].into_iter(),
            (Some(StepKind::EO), StepKind::DR) => {
                let dr_table = tables.dr().expect("DR table required");
                if config.params.contains_key("triggers") {
                    info!("Found explicitly defined DR triggers without RZP. Adding RZP step with default settings.");
                    let mut rzp_config = StepConfig::new(StepKind::RZP);
                    rzp_config.quality = config.quality;
                    vec![rzp::from_step_config(rzp_config), dr_trigger::from_step_config(dr_table, config)].into_iter()
                } else {
                    vec![dr::from_step_config(dr_table, config)].into_iter()
                }
            }
            (Some(StepKind::RZP), StepKind::DR) => {
                let dr_table = tables.dr().expect("DR table required");
                if !config.params.contains_key("triggers") {
                    warn!("RZP without defining triggers is pointless and slower. Consider deleting the RZP step or adding explicit DR triggers.");
                    vec![dr::from_step_config::<CubieCube>(dr_table, config)].into_iter()
                } else {
                    vec![dr_trigger::from_step_config(dr_table, config)].into_iter()
                }
            }
            (Some(StepKind::DR), StepKind::HTR)   => vec![htr::from_step_config::<CubieCube>(tables.htr().expect("HTR table required"), config)].into_iter(),
            (Some(StepKind::HTR), StepKind::FR)   => vec![fr::from_step_config::<CubieCube>(tables.fr().expect("FR table required"), config)].into_iter(),
            (Some(StepKind::HTR), StepKind::FRLS)  => vec![fr::from_step_config_no_slice::<CubieCube>(tables.fr_leave_slice().expect("FRLeaveSlice table required"), config)].into_iter(),
            (Some(StepKind::FR), StepKind::FIN)   => vec![finish::from_step_config_fr::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config)].into_iter(),
            (Some(StepKind::FRLS), StepKind::FIN)   => vec![finish::from_step_config_fr_leave_slice::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config)].into_iter(),
            (None, x) => vec![Err(format!("{:?} is not supported as a first step", x))].into_iter(),
            (Some(x), y) => vec![Err(format!("Unsupported step order {:?} > {:?}", x, y))].into_iter(),
        })
        .collect();

    let steps = if let Ok(steps) = steps {
        steps
    } else if let Err(err) = steps {
        error!("{err}");
        return;
    } else {
        unreachable!()
    };

    let first_step: Box<dyn Iterator<Item = Solution>> = Box::new(vec![Solution::new()].into_iter());

    let solutions = steps.iter()
        .fold(first_step, |acc, (step, search_opts)|{
            debug!("Step {} with options {:?}", step.name(), search_opts);
            let next = step::next_step(acc, step, search_opts.clone(), cube.clone())
                .zip(0..)
                .take_while(|(sol, count)| search_opts.step_limit.map(|limit| limit > *count).unwrap_or(true))
                .map(|(sol, _)|sol);
            Box::new(next)
        });

    let time = Instant::now();

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions
        .skip_while(|alg| alg.len() < cli.min)
        .take_while(|alg| cli.max.map_or(true, |max| alg.len() <= max)));


    // For e.g. FR the direction of the last move always matters so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.is_half_turn_invariant()).unwrap_or(true);
    if !cli.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::filter_eo_last_moves_pure(&alg.clone().into())));
    }

    //We already generate a mostly duplicate iterator, but sometimes the same solution is valid for different stages and that can cause duplicates.
    let solutions = stream::distinct_algorithms(solutions);

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
