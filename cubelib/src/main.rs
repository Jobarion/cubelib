use std::str::FromStr;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use cubelib::algs::Algorithm;
use cubelib::defs::NissSwitchType;
use cubelib::puzzles::c222::{Cube222, Turn222};
use cubelib::puzzles::puzzle::{ApplyAlgorithm, TurnableMut};
use cubelib::puzzles::pyraminx::{Pyraminx, PyraminxTip, PyraminxTurn};
use cubelib::puzzles::pyraminx::steps::finish::{direct_finish, tips, TipStepVariant};

use cubelib::puzzles::pyraminx::steps::tables::PruningTablesPyraminx;
use cubelib::solver::solve_steps;
use cubelib::steps::step::{DefaultStepOptions, first_step, StepVariant};

pub fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    let mut c222 = Cube222::default();

    let alg = Algorithm::<Turn222>::from_str("R B' R L B U' L' U' B' U B' l' r' b").unwrap();
    pyra.apply_alg(&alg);
    println!("{alg}");

    let mut pt = PruningTablesPyraminx::new();
    pt.gen_finish_no_tips();

    let steps = vec![
        (tips(), DefaultStepOptions::new(0, 4, NissSwitchType::Never, None)),
        (direct_finish(&pt.fin().expect("Pruning table required")), DefaultStepOptions::new(0, 11, NissSwitchType::Never, None)),
    ];
    let solution: Algorithm<PyraminxTurn> = solve_steps(pyra, &steps).next().map(|s|s.into()).unwrap();
    println!("{:?}", solution.split_tips_and_no_tips());
}