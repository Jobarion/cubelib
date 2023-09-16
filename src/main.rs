extern crate core;

use std::cmp::{max, min};
use std::collections::HashSet;
use std::ops::Add;
use std::{mem, primitive, thread};
use std::str::FromStr;
use std::time::{Duration, Instant};
use itertools::Itertools;
use crate::algs::Algorithm;
use crate::coord::{COUDCoord, Coord, CPCoord, EOCoordAll, EOCoordUD, DRUDEOFBCoord, EPCoord, UDSliceUnsortedCoord, EOCoordFB};
use crate::cube::{Axis, Cube, Face, Move, NewSolved, Transformation, Turn, Turnable};
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search::{ALL_MOVES, dfs_iter, MoveSkipTracker, NissType, SearchOptions};
use crate::step::{first_step, StepVariant};
use crate::eo::{EOCount, EOStepTable};
use crate::lookup_table::{PruningTable};
use crate::moveset::TransitionTable;
use crate::stream::DFSAlgIter;
// use crate::cubie::CubieCube;

mod cube;
mod cubie;
mod eo;
mod algs;
mod df_search;
mod dr;
mod alignment;
mod coord;
mod lookup_table;
mod co;
mod moveset;
mod stream;
mod htr;
mod step;

fn main() {
    let time = Instant::now();

    let eofb_table = lookup_table::generate(&eo::EO_FB_MOVESET, &|c: &CubieCube| EOCoordFB::from(&c.edges));
    let drud_eofb_table = lookup_table::generate(&dr::DR_UD_EO_FB_MOVESET, &|c: &CubieCube| DRUDEOFBCoord::from(c));

    println!("Took {}ms", time.elapsed().as_millis());


    let mut cube = cubie::CubieCube::new_solved();

    let scramble = Algorithm { normal_moves: algs::parse_algorithm("R' U' F U F2 D U2 L2 D R2 U' L2 R U' F2 L' U2 L' F' L2 U2 L F R' U' F"), inverse_moves: vec![] };
    cube.apply_alg(&scramble);

    // let eo_stage = eofb_stage();
    // let dr_stage = drud_eofb_stage();

    {
        let eo_step = eo::eo(&eofb_table);
        let dr_step = dr::dr(&drud_eofb_table);
        let eo_solutions = step::first_step(&eo_step, SearchOptions::new(0, 5, NissType::During), cube.edges.clone());

        let dr_solutions = step::next_step(eo_solutions, &dr_step, SearchOptions::new(4, 14, NissType::None), cube.clone());


        // mem::drop(eo_step);
        for a in dr_solutions.take(20) {
            println!("{a}");
        }

    }



    // eo::eoud()


    // let eo_stage = EOStepTable::new_fb(&eofb_table);
    // dfs_iter(&eo_stage, cube.edges, SearchOptions::new(0, 5, NissType::During))
    //     .expect("Cube not ready for stage")
    //     .for_each(|alg| println!("{alg}"))
    // let mut eo_stage = dfs_table_heuristic(SearchOptions::new(&eo_stage, 0, 5, NissType::During), &eofb_table, cube.edges)

        // .skip(24)
        ;

    // for x in eo_stage {
    //     println!("{x}");
    // }


    // let dr_stage = stream::next_stage(eo_stage, |alg, depth|{
    //     let mut eo_cube = cube.clone();
    //     eo_cube.apply_alg(&alg);
    //     dfs_table_heuristic(SearchOptions::new(&dr_stage, depth, depth, NissType::None), &drud_eofb_table, eo_cube)
    //         .map(move |dr|alg.clone().add(dr))
    // }).filter(|alg| eo::filter_eo_last_moves_pure(&alg));
    // dr_stage.take(10).for_each(|alg|println!("{alg} {}", alg.len()));
    //
    // println!("\n\n\n");
    //
    // let solutions: Vec<Algorithm> = dfs_table_heuristic(&eo::EO_FB_MOVESET, &eofb_table, cube.edges, 0, 20, true)
    //     .filter(|alg| eo::filter_eo_last_moves_pure(&alg))
    //     .take_while(|alg| alg.len() <= 5)
    //     .flat_map(|eo| {
    //         let mut eo_cube = cube.clone();
    //         let eo_clone = eo.clone();
    //         eo_cube.apply(&eo);
    //         dfs_table_heuristic(&dr::EO_FB_DR_LR_MOVESET, &eofb_drlr_table, eo_cube, 0, 20, false)
    //             .filter(|alg| eo::filter_eo_last_moves_pure(&alg))
    //             .take_while(move |dr|dr.len() + eo_clone.len() <= 14)
    //             .take(1)
    //             .map(move |dr|eo.clone().add(dr))
    //     })
    //     .sorted_by(|dr1, dr2|dr1.len().cmp(&dr2.len()))
    //     .collect();
    // for dr in solutions {
    //     println!("{dr} {}", dr.len());
    // }

    // println!("Using UD-EO {}", eo);
    //
    // println!("{:?}", EOUDDRFBCoord::from(&cube));
    //
    // cube.apply(&eo);
    //
    // let dr = dr::eo_fb_dr_ud_iter_table_heuristic(&cube, &eofb_drlr_table)
    //     .next()
    //     .unwrap();
    //
    // println!("Using DR {}", dr);
    //
    // cube.apply(&dr);
    //
    // println!("{}", cube);
    //
    // println!("{:?}", EOUDDRFBCoord::from(&cube));
    // println!("{:?}", COCoordUD::from(&cube.corners));

    println!("Took {}ms", time.elapsed().as_millis());
}
