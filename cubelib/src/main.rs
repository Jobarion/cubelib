use std::str::FromStr;
use std::time::Instant;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::Cube333;
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::defs::NissSwitchType;
use cubelib::solver_new::create_worker;
use cubelib::solver_new::dr::DRStep;
use cubelib::solver_new::eo::EOStep;
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
            .max_length(4)
            .niss(NissSwitchType::Always)
            .build(),
        EOStep::builder()
            .max_length(5)
            .min_length(5)
            .niss(NissSwitchType::Before)
            .build()
    ]);

    let dr_step = DRStep::builder()
        .max_absolute_length(13)
        .niss(NissSwitchType::Never)
        .build();

    let htr_step = HTRStep::builder()
        .niss(NissSwitchType::Never)
        .build();

    let cube = Algorithm::from_str("D2 F R' U2 F2 R2 D2 B2 L B2 R' B2 L B2 D' B D2 U' R F' L'").unwrap().into();
    let steps = StepGroup::sequential_with_predicates(vec![eo_step, dr_step, htr_step], vec![FilterLastMoveNotPrime::new()]);

    let (mut worker, receiver) = create_worker(cube, steps);
    worker.start();

    let start = Instant::now();
    for i in 0..2 {
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

