use std::str::FromStr;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::{Cube333, Transformation333};
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::defs::NissSwitchType;
use cubelib::solver::solution::Solution;
use cubelib::solver_new::dr::{DROptions, DRStep, DRStepOptions};
use cubelib::solver_new::eo::{EOOptions, EOStep, EOStepOptions};
use cubelib::solver_new::group::{Parallel, Sequential};
use cubelib::solver_new::thread_util::ToWorker;
use cubelib::solver_new::step::bounded_channel;
use cubelib::steps::tables::{ArcPruningTable333, PruningTables333};

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    let (mut tx0, mut rc0) = bounded_channel(1);
    let (mut tx1, mut rc1) = bounded_channel(1);
    let eo_opts: EOOptions = EOOptions::builder()
        .max_length(3)
        .options(EOStepOptions::builder()
            .niss(NissSwitchType::Never)
            .build())
        .build();

    let dr_opts: DROptions = DROptions::builder()
        .options(DRStepOptions::builder()
            .niss(NissSwitchType::Never)
            .build())
        .build();

    let eo_step = EOStep::new(eo_opts);
    let dr_step = DRStep::new(dr_opts);

    tx0.send(Solution::new()).unwrap();
    drop(tx0);
    let mut cube = Cube333::default();
    // cube.apply_alg(&Algorithm::from_str("R' U' F L' R D2 R' F2 R' F2 U2 D' R' F' U2 L' F' L2 F' U' B2 U2 R' U' F").unwrap());
    cube.apply_alg(&Algorithm::from_str("R' F'").unwrap());
    let mut worker = Sequential::new(vec![Box::new(eo_step), dr_step]).to_worker(cube, rc0, tx1);
    worker.start();

    thread::spawn(move ||{
        for _ in 0..5 {
            let solution = if let Ok(s) = rc1.recv() {
                s
            } else {
                break
            };
            println!("{solution:?}");
        }
    }).join().unwrap();
    worker.stop().unwrap().join().unwrap();
    // dr_worker.stop().unwrap().join().unwrap();
    // eo_worker.stop().unwrap().join().unwrap();
}

