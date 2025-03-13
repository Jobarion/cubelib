use std::collections::HashMap;
use std::str::FromStr;
use std::time::Instant;
use std::vec;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::turn::CubeAxis;
use cubelib::defs::NissSwitchType;
use cubelib::solver_new::create_worker;
use cubelib::solver_new::dr::{DRStep, RZPStep};
use cubelib::solver_new::eo::EOStep;
use cubelib::solver_new::finish::HTRFinishStep;
use cubelib::solver_new::group::StepGroup;
use cubelib::solver_new::htr::HTRStep;
use cubelib::solver_new::util_steps::FilterLastMoveNotPrime;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    let eo_step = StepGroup::parallel(vec![
        EOStep::builder()
            .max_length(7)
            .max_absolute_length(7)
            .niss(NissSwitchType::Always)
            .eo_axis(vec![CubeAxis::FB])
            .build(),
        // EOStep::builder()
        //     .max_length(5)
        //     .min_length(5)
        //     .niss(NissSwitchType::Before)
        //     .build()
    ]);

    let dr_step = DRStep::builder()
        .max_absolute_length(15)
        .niss(NissSwitchType::Before)
        .axis(HashMap::from([(CubeAxis::UD, vec![CubeAxis::FB])]))
        .build();

    let cube = Algorithm::from_str("D2 F R' U2 F2 R2 D2 B2 L B2 R' B2 L B2 D' B D2 U' R F' L'").unwrap().into();
    let steps = StepGroup::sequential_with_predicates(vec![eo_step, dr_step], vec![FilterLastMoveNotPrime::new()]);

    let (mut worker, receiver) = create_worker(cube, steps);
    worker.start();

    let start = Instant::now();
    for i in 0..10 {
        let solution = if let Ok(s) = receiver.recv() {
            s
        } else {
            break
        };
        println!("{i}\n{solution}");
    }
    println!("Took {}ms", start.elapsed().as_millis());
    drop(receiver);
    worker.stop().unwrap().join().unwrap();
}
