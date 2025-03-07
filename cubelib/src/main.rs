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
use cubelib::solver_new::create_worker;
use cubelib::solver_new::dr::{DROptions, DRStep, DRStepOptions};
use cubelib::solver_new::eo::{EOOptions, EOStep, EOStepOptions};
use cubelib::solver_new::htr::{HTROptions, HTRStep, HTRStepOptions};
use cubelib::solver_new::util_steps::{FilterDup, FilterLastMoveNotPrime};
use cubelib::steps::util::{SUBSETS_0C0, SUBSETS_2C3, SUBSETS_4A1, SUBSETS_4A2, SUBSETS_4B2};

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    let eo_opts: EOOptions = EOOptions::builder()
        .max_length(5)
        .options(EOStepOptions::builder()
            .eo_axis(vec![CubeAxis::FB])
            .niss(NissSwitchType::Never)
            .build())
        .build();

    let dr_opts: DROptions = DROptions::builder()
        .options(DRStepOptions::builder()
            .niss(NissSwitchType::Never)
            .subsets(vec![
                // SUBSETS_0C0,
                SUBSETS_4A1,
                // SUBSETS_4A2,
                // SUBSETS_4B2,
                // SUBSETS_2C3,
            ].into_iter().flat_map(|x|x.into_iter()).cloned().collect())
            .dr_eo_axis(HashMap::from([(CubeAxis::UD, vec![CubeAxis::FB])]))
            .build())
        .build();

    let htr_opts: HTROptions = HTROptions::builder()
        .options(HTRStepOptions::builder()
            .niss(NissSwitchType::Never)
            .build()
        )
        .build();

    let eo_step = EOStep::new(eo_opts);
    let dr_step = DRStep::new(dr_opts);
    let htr_step = HTRStep::new(htr_opts);

    let mut cube = Cube333::default();
    cube.apply_alg(&Algorithm::from_str("L2 F U2 F' U2 F' R2 F2 L2 F' D R' U2 B2 U L2 B' U2 R2").unwrap());

    let (mut worker, receiver) = create_worker(cube, vec![eo_step, dr_step, htr_step, Box::new(FilterLastMoveNotPrime), Box::new(FilterDup)]);

    worker.start();

    let start = Instant::now();
    for i in 0..5 {
        let solution = if let Ok(s) = receiver.recv() {
            s
        } else {
            println!("Terminated");
            break
        };
        println!("{i}\n{solution}");
    }
    println!("Took {}ms", start.elapsed().as_millis());
    drop(receiver);
    worker.stop().unwrap().join().unwrap();
}

