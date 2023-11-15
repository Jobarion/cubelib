use std::str::FromStr;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use cubelib::algs::Algorithm;
use cubelib::defs::NissSwitchType;
use cubelib::puzzles::c222::{Cube222, Turn222};
use cubelib::puzzles::c222::steps::tables::PruningTables222;
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

    let mut pt = PruningTables222::new();
    pt.gen_fin();
}