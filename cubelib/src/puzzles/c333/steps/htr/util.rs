use std::cmp::max;
use std::collections::HashMap;
use std::str::FromStr;
use itertools::Itertools;
use log::debug;
use crate::algs::Algorithm;
use crate::puzzles::c333::{Cube333, Turn333};
use crate::puzzles::c333::steps::htr::coords::HTRDRUDCoord;
use crate::puzzles::c333::steps::htr::htr_config::{HTR_DR_UD_MOVESET, HTRPruningTable};
use crate::puzzles::cube::Direction;
use crate::puzzles::puzzle::{ApplyAlgorithm, InvertibleMut, TurnableMut};

const SUBSET_GENERATORS: [&str; 48] = [
    "",
    "U R2 F2 R2 U",
    "U R2 L2 D",
    "U R2 L2 F2 R2 F2 D",
    "U R2 L2 F2 B2 U",
    "U R2 L2 U F2 B2 D",
    "U R2 L2 U R2 U",
    "U",
    "U R2 F2 R2 F2 U",
    "U R2 U",
    "U R2 F2 U",
    "U R2 U R2 B2 R2 U' R2 U",
    "D B2 D' F2 B2 D' F2 D",
    "U R2 U2 F2 U",
    "U F2 U2 R2 B2 U' L2 B2 D",
    "U R2 U R2 U",
    "U L2 U F2 U",
    "U L2 D R2 F2 B2 U",
    "U B2 L2 U B2 L2 U2 B2 D",
    "U R2 F2 U R2 U2 F2 U",
    "U B2 U R2 U2 F2 D",
    "U B2 U' L2 U2 B2 D",
    "U R2 U2 F2 U' R2 U2 R2 F2 U",
    "U R2 U2 F2 U R2 U2 F2 U",
    "U R2 U2 F2 U R2 U2 R2 B2 D",
    "U L2 U2 F2 U B2 U2 R2 U",
    "U R2 U2 F2 U' L2 U2 R2 F2 D",
    "U L2 D' R2 D L2 U",
    "U R2 U' R2 U R2 U",
    "U R2 U' L2 D R2 D",
    "U' B2 D' L2 B2 U' R2 U",
    "U R2 L2 B2 U R2 U' F2 U",
    "U R2 U R2 U2 B2 U B2 U",
    "U L2 U B2 U2 R2 U L2 U",
    "U B2 U F2 U2 R2 U R2 D",
    "U' R2 U L2 U2 L2 B2 U' B2 D",
    "U' R2 U R2 U2 B2 U F2 R2 L2 U",
    "U R2 U B2 U2 R2 F2 B2 U' B2 U",
    "U' F2 U F2 U2 R2 U' R2 U",
    "U' B2 U F2 U2 R2 U' R2 U",
    "U L2 U L2 U' L2 B2 U' B2 U",
    "U L2 U R2 U' R2 U R2 U",
    "D R2 U L2 D' R2 D L2 U",
    "U L2 U F2 U' R2 U B2 U",
    "U R2 U L2 F2 U B2 U' L2 U",
    "U' L2 U L2 U' R2 U R2 U",
    "U' R2 U F2 U' F2 U B2 D",
    "U' R2 U F2 U' F2 U F2 U"
];

pub fn gen_niss_table(htr_table: &mut HTRPruningTable) {
    let cube = Cube333::default();
    for (alg_str, id) in SUBSET_GENERATORS.iter().zip(1..) {
        let alg = Algorithm::from_str(alg_str).unwrap();
        let mut c = cube.clone();
        let min_moves = min_niss_moves(&alg);
        debug!("Generating NISS table for subgroup: {id} ({alg}), min moves: {min_moves}");

        c.apply_alg(&alg);
        fill_table(htr_table, alg, min_moves);
    }
}

fn min_niss_moves(alg: &Algorithm<Turn333>) -> u8 {
    let mut max_half_turns = 0;
    let mut current_half_turns = 0;
    for turn in alg.normal_moves.iter() {
        if turn.dir == Direction::Half {
            current_half_turns += 1;
            max_half_turns = max(current_half_turns, max_half_turns);
        } else {
            current_half_turns = 0;
        }
    }
    alg.normal_moves.len() as u8 - max_half_turns
}

fn gen_coset_0() -> Vec<Cube333> {
    let mut checked = HashMap::new();
    let mut to_check: Vec<Cube333> = vec![Cube333::default()];
    let mut check_next: Vec<Cube333> = vec![];

    loop {
        for cube in to_check {
            for cube in HTR_DR_UD_MOVESET.aux_moves.iter().cloned()
                .map(|m|{
                    let mut cube = cube.clone();
                    cube.turn(m);
                    cube
                }) {
                let coord = HTRDRUDCoord::from(&cube);
                if checked.contains_key(&coord) {
                    continue;
                }
                checked.insert(coord, cube);
                check_next.push(cube);
            }
        }
        if check_next.is_empty() {
            break;
        }
        to_check = check_next;
        check_next = vec![];
    }
    checked.values().cloned().collect_vec()
}

fn fill_table(htr_table: &mut HTRPruningTable, alg: Algorithm<Turn333>, niss_bound: u8) {
    if niss_bound == 0 {
        return;
    }
    let mut to_check: Vec<Cube333> = gen_coset_0()
        .into_iter()
        .flat_map(|c|{
            vec![c].into_iter()
                .flat_map(|mut a| {
                    let b = a.clone();
                    a.turn(Turn333::U);
                    a.turn(Turn333::D);
                    vec![a, b].into_iter()
                })
                .map(|mut c|{
                    c.apply_alg(&alg);
                    c
                })
        })
        .collect_vec();
    let mut check_next: Vec<Cube333> = vec![];
    loop {
        debug!("To check: {}", to_check.len());
        for cube in to_check.iter().cloned().flat_map(|mut a|{
            let b = a.clone();
            a.invert();
            vec![a, b].into_iter()
        }) {
            for cube in HTR_DR_UD_MOVESET.aux_moves.iter().cloned()
                .map(|m|{
                    let mut cube = cube.clone();
                    cube.turn(m);
                    cube
                }) {
                let coord = HTRDRUDCoord::from(&cube);
                let (normal, niss) = htr_table.get(coord);
                if niss != 0 {
                    continue;
                }
                htr_table.set(coord, normal | (niss_bound << 4));
                check_next.push(cube);
            }
        }
        if check_next.is_empty() {
            break;
        }
        to_check = check_next;
        check_next = vec![];
    }
}