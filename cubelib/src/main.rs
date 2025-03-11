use std::str::FromStr;
use std::time::Instant;
use std::vec;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::turn::CubeAxis;
use cubelib::defs::NissSwitchType;
use cubelib::solver_new::create_worker;
use cubelib::solver_new::dr::{DRStep, RZPBuilder};
use cubelib::solver_new::eo::EOStep;
use cubelib::solver_new::group::StepGroup;
use cubelib::solver_new::htr::HTRStep;
use cubelib::solver_new::util_steps::FilterLastMoveNotPrime;
use cubelib::steps::dr::rzp_config::RZPStep;
fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();

    let eo_step = StepGroup::parallel(vec![
        EOStep::builder()
            .max_length(4)
            .eo_axis(vec![CubeAxis::LR])
            .niss(NissSwitchType::Never)
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
        .rzp(RZPBuilder::new()
            .max_length(3)
            .max_absolute_length(6)
        )
        .triggers(vec![Algorithm::from_str("R U2 R").unwrap(), Algorithm::from_str("R").unwrap()])
        .build();

    let htr_step = HTRStep::builder()
        .niss(NissSwitchType::Never)
        .build();

    let cube = Algorithm::from_str("D2 F R' U2 F2 R2 D2 B2 L B2 R' B2 L B2 D' B D2 U' R F' L'").unwrap().into();
    let steps = StepGroup::sequential_with_predicates(vec![eo_step, dr_step, htr_step], vec![FilterLastMoveNotPrime::new()]);

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
