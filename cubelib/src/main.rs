use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use itertools::Itertools;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::{Cube333, CubeAxis};
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::defs::NissSwitchType;
use cubelib::solver::solution::Solution;
use cubelib::solver_new::dr::{DROptions, DRStep, DRStepOptions};
use cubelib::solver_new::eo::{EOOptions, EOStep, EOStepOptions};
use cubelib::solver_new::group::Sequential;
use cubelib::solver_new::step::{bounded_channel, create_worker};
use cubelib::solver_new::thread_util::ToWorker;
use cubelib::solver_new::util_steps::{FilterDup, FilterLastMoveNotPrime};
use cubelib::steps::util::{expand_subset_name, SUBSETS_0C0, SUBSETS_2C3, SUBSETS_2C4, SUBSETS_4A1, SUBSETS_4A2, SUBSETS_4B2};

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    let eo_opts: EOOptions = EOOptions::builder()
        .max_length(20)
        .max_absolute_length(5)
        .options(EOStepOptions::builder()
            .niss(NissSwitchType::Never)
            .build())
        .build();

    let dr_opts: DROptions = DROptions::builder()
        .max_length(15)
        .max_absolute_length(15)
        .options(DRStepOptions::builder()
            .niss(NissSwitchType::Never)
            .subsets(vec![
                SUBSETS_0C0,
                SUBSETS_4A1,
                SUBSETS_4A2,
                SUBSETS_4B2,
                SUBSETS_2C3,
            ].into_iter().flat_map(|x|x.into_iter()).cloned().collect())
            .dr_eo_axis(HashMap::from([(CubeAxis::LR, vec![CubeAxis::FB])]))
            .build())
        .build();

    let eo_step = EOStep::new(eo_opts);
    let dr_step = DRStep::new(dr_opts);

    let mut cube = Cube333::default();
    cube.apply_alg(&Algorithm::from_str("R' U' F L' R D2 R' F2 R' F2 U2 D' R' F' U2 L' F' L2 F' U' B2 U2 R' U' F").unwrap());

    let (mut worker, receiver) = create_worker(cube, vec![eo_step, dr_step, Box::new(FilterLastMoveNotPrime), Box::new(FilterDup)]);

    // let mut worker = Sequential::new(vec![eo_step, dr_step, Box::new(FilterLastMoveNotPrime), Box::new(FilterDup)]).to_worker(cube, rc0, tx1);
    worker.start();

    let start = Instant::now();
    for i in 0..100 {
        let solution = if let Ok(s) = receiver.recv() {
            s
        } else {
            break
        };
        println!("{i} {solution:?}");
    }
    println!("Took {}ms", start.elapsed().as_millis());
    drop(receiver);
    worker.stop().unwrap().join().unwrap();
}

