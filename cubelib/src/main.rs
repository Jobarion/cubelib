use std::str::FromStr;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use itertools::Itertools;
use log::LevelFilter;
use simple_logger::SimpleLogger;

use cubelib::algs::Algorithm;
use cubelib::cube::Cube333;
use cubelib::cube::turn::ApplyAlgorithm;
use cubelib::defs::NissSwitchType;
use cubelib::solver::solution::Solution;
use cubelib::solver_new::dr::{DROptions, DRStep, DRStepOptions};
use cubelib::solver_new::eo::{EOOptions, EOStep, EOStepOptions};
use cubelib::solver_new::group::Sequential;
use cubelib::solver_new::step::{Step, ToWorker, Worker};
use cubelib::steps::tables::{ArcPruningTable333, PruningTables333};

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
    let (mut tx0, mut rc0) = std::sync::mpsc::sync_channel(1);
    let (mut tx1, mut rc1) = std::sync::mpsc::sync_channel(1);
    // let (mut tx2, mut rc2) = std::sync::mpsc::sync_channel(1);
    let eo_opts: EOOptions = EOOptions::builder()
        .max_length(4)
        .options(EOStepOptions::builder()
            .niss(NissSwitchType::Always)
            .build())
        .build();

    let dr_opts: DROptions = DROptions::builder()
        .options(DRStepOptions::builder()
            .niss(NissSwitchType::Never)
            .build())
        .build();

    let mut pt = PruningTables333::new();
    pt.gen_eo();
    pt.gen_dr();
    let apt: ArcPruningTable333 = pt.into();

    let eo_step = EOStep::new(eo_opts, apt.eo.expect("EO Table required").clone());
    let dr_step = DRStep::new(dr_opts, apt.dr.expect("DR Table required").clone());

    tx0.send(Solution::new()).unwrap();
    drop(tx0);
    let mut cube = Cube333::default();
    cube.apply_alg(&Algorithm::from_str("R' U' F L' R D2 R' F2 R' F2 U2 D' R' F' U2 L' F' L2 F' U' B2 U2 R' U' F").unwrap());
    let mut worker = Sequential::new(vec![Box::new(eo_step), Box::new(dr_step)]).to_worker(cube, rc0, tx1);
    // let mut eo_worker = x.to_worker(cube.clone(), rc0, tx1);
    // let mut eo_worker = eo_step.to_worker(cube.clone(), rc0, tx1);
    // let mut dr_worker = dr_step.to_worker(cube.clone(), rc1, tx2);
    // eo_worker.start();
    // dr_worker.start();
    worker.start();

    thread::spawn(move ||{
        for _ in 0..1 {
            let solution = if let Ok(s) = rc1.recv() {
                s
            } else {
                break
            };
            println!("{solution}");
        }
    });
    sleep(Duration::from_secs(5));
    // dr_worker.stop().unwrap().join().unwrap();
    // eo_worker.stop().unwrap().join().unwrap();
}

