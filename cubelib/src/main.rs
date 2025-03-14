use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use std::vec;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::{Cube333, Transformation333};
use cubelib::cube::turn::{ApplyAlgorithm, CubeAxis, TransformableMut};
use cubelib::defs::NissSwitchType;
use cubelib::solver_new::create_worker;
use cubelib::solver_new::dr::{DRStep, RZPStep};
use cubelib::solver_new::eo::EOStep;
use cubelib::solver_new::finish::HTRFinishStep;
use cubelib::solver_new::group::StepGroup;
use cubelib::solver_new::htr::{HTR_TABLES, HTRStep};
use cubelib::solver_new::util_steps::FilterLastMoveNotPrime;
use cubelib::steps::htr::coords::HTRDRUDCoord;
use cubelib::steps::util::DR_SUBSETS;

fn main() {
    // let mut cube: Cube333 = Algorithm::from_str("U' R F2 R2 B2 L' R2 F2 L B2 R' U2 F2 U' R' B' U B F L' F'").unwrap().into();
    // cube.apply_alg(&Algorithm::from_str("R' F' B2 L B2 D2 B U2 B' L2 B' R2 U R2 F R2 B").unwrap());
    // cube.transform(Transformation333::X);
    // println!("{cube}");
    // println!("{:?}", HTRDRUDCoord::from(&cube));

    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    let eo_step = StepGroup::parallel(vec![
        EOStep::builder()
            .max_length(4)
            .max_absolute_length(4)
            .niss(NissSwitchType::Always)
            .build(),
        // EOStep::builder()
        //     .max_length(5)
        //     .min_length(5)
        //     .niss(NissSwitchType::Before)
        //     .build()
    ]);

    let dr_step = DRStep::builder()
        .max_absolute_length(14)
        .max_length(12)
        .niss(NissSwitchType::Before)
        .triggers(vec![Algorithm::from_str("R").unwrap()])
        .rzp(RZPStep::builder().max_length(0).build())
        .build();

    let htr_step = HTRStep::builder()
        .niss(NissSwitchType::Always)
        .build();

    let mut cube: Cube333 = Algorithm::from_str("U' R F2 R2 B2 L' R2 F2 L B2 R' U2 F2 U' R' B' U B F L' F'").unwrap().into();
    cube.transform(Transformation333::X);
    println!("{:?}", HTR_TABLES.0.get(HTRDRUDCoord::from(&cube)));
    println!("{}", DR_SUBSETS[HTR_TABLES.1.get(HTRDRUDCoord::from(&cube)) as usize]);

    let steps = StepGroup::sequential_with_predicates(vec![eo_step, dr_step, htr_step], vec![FilterLastMoveNotPrime::new()]);

    let (mut worker, receiver) = create_worker(cube, steps);
    worker.start();

    let start = Instant::now();
    for i in 0..4 {
        let solution = if let Ok(s) = receiver.recv() {
            s
        } else {
            break
        };
        if solution.len() > 17 {
            break;
        }
        println!("{i}\n{solution}");
    }
    println!("Took {}ms", start.elapsed().as_millis());
    drop(receiver);
    worker.stop().unwrap().join().unwrap();
}
