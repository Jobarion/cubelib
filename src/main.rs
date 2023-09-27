extern crate core;

use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use itertools::Itertools;
use log::{debug, error, info, LevelFilter};
use simple_logger::SimpleLogger;

use cubelib::algs::{Algorithm, Solution};
use cubelib::cli::{Cli, StepKind};
use cubelib::cube::{ApplyAlgorithm, NewSolved};
use cubelib::cubie::CubieCube;
use cubelib::df_search::SearchOptions;
use cubelib::steps::{dr, eo, finish, fr, htr, step};
use cubelib::steps::step::Step;
use cubelib::stream;
use cubelib::tables::PruningTables;

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

    let steps: Result<Vec<(Step<CubieCube>, SearchOptions)>, String> = steps.into_iter()
        .map(|(config, previous)| match (previous, config.kind) {
            (None, StepKind::EO) => eo::from_step_config::<CubieCube>(tables.eo().expect("EO table required"), config),
            (Some(StepKind::EO), StepKind::DR)   => dr::from_step_config::<CubieCube>(tables.dr().expect("DR table required"), config),
            (Some(StepKind::DR), StepKind::HTR)   => htr::from_step_config::<CubieCube>(tables.htr().expect("HTR table required"), config),
            (Some(StepKind::HTR), StepKind::FR)   => fr::from_step_config::<CubieCube>(tables.fr().expect("FR table required"), config),
            (Some(StepKind::HTR), StepKind::FRLS)  => fr::from_step_config_no_slice::<CubieCube>(tables.fr_leave_slice().expect("FRLeaveSlice table required"), config),
            (Some(StepKind::FR), StepKind::FIN)   => finish::from_step_config_fr::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config),
            (Some(StepKind::FRLS), StepKind::FIN)   => finish::from_step_config_fr_leave_slice::<CubieCube>(tables.fr_finish().expect("FRFinish table required"), config),
            (None, x) => Err(format!("{:?} is not supported as a first step", x)),
            (Some(x), y) => Err(format!("Unsupported step order {:?} > {:?}", x, y)),
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

    let mut solutions = steps.iter()
        .fold(first_step, |acc, (step, search_opts)|{
            debug!("Step {} with options {:?}", step.name(), search_opts);
            Box::new(step::next_step(acc, step, search_opts.clone(), cube.clone())
                .zip(0..)
                .take_while(|(sol, count)| search_opts.step_limit.map(|limit| limit > *count).unwrap_or(true))
                .map(|(sol, _)|sol))
        });

    let time = Instant::now();

    let mut solutions: Box<dyn Iterator<Item = Solution>> = Box::new(solutions
        .skip_while(|alg| alg.len() < cli.min)
        .take_while(|alg| cli.max.map(|max| alg.len() <= max).unwrap_or(true)));


    // For FR the direction of the last move always matters so we can't filter if we're doing FR
    let can_filter_last_move = steps.last().map(|(s, _)| s.is_half_turn_invariant()).unwrap_or(true);
    if !cli.all_solutions && can_filter_last_move {
        solutions = Box::new(solutions
            .filter(|alg| eo::filter_eo_last_moves_pure(&alg.clone().into())));
    }

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
