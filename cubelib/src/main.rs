use std::str::FromStr;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use cubelib::algs::Algorithm;
use cubelib::cube::Cube333;
use cubelib::cube::turn::CubeAxis;
use cubelib::solver::solution::ApplySolution;
use cubelib::solver_new::create_worker;
use cubelib::solver_new::eo::EOStep;
use cubelib::steps::eo::coords::BadEdgeCount;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
    let mut cube: Cube333 = Algorithm::from_str("R' U' F U2 L2 R2 U' L2 D' U R2 B U2 R B' L' F2 U2 L D R F' U2 R' U' F (R' D' F U) (L F) (R2 F2 R B)").unwrap().into();
    println!("{:?}", cube.count_bad_edges_fb());
    println!("{:?}", cube.count_bad_edges_ud());
    println!("{:?}", cube.count_bad_edges_lr());
    println!("{cube}");
    let eo = EOStep::builder()
        .min_length(0)
        .eo_axis(vec![CubeAxis::FB])
        .build();
    let (mut worker, rc) = create_worker(cube.clone(), eo);
    worker.start();
    let next = rc.recv().unwrap();
    println!("{next}");
    cube.apply_solution(&next);
    println!("{:?}", cube.count_bad_edges_fb());
    println!("{:?}", cube.count_bad_edges_ud());
    println!("{:?}", cube.count_bad_edges_lr());
    println!("{cube}");

    drop(rc);
    worker.stop();
}