use std::cmp::max;
use std::collections::HashMap;
use std::str::FromStr;
use itertools::Itertools;
use log::{debug, trace};
use crate::algs::Algorithm;
use crate::puzzles::c333::{Cube333, Turn333};
use crate::puzzles::c333::steps::htr::coords::HTRDRUDCoord;
use crate::puzzles::c333::steps::htr::htr_config::{HTR_DR_UD_MOVESET, HTRPruningTable, HTRSubsetTable};
use crate::puzzles::cube::Direction;
use crate::puzzles::puzzle::{ApplyAlgorithm, InvertibleMut, TurnableMut};
use crate::steps::coord::Coord;

pub type Subset = crate::puzzles::c333::util::Subset;
pub const HTR_SUBSETS: [Subset; 48] = crate::puzzles::c333::util::HTR_SUBSETS;

pub fn gen_subset_tables(htr_table: &mut HTRPruningTable) -> HTRSubsetTable {
    let mut subset_table = HTRSubsetTable::new(false);

    let table_size = HTRDRUDCoord::size();
    let mut total_checked = 0;

    for (subset, id) in HTR_SUBSETS.iter().zip(1..) {
        debug!("Generating NISS table for subset: {id}. {subset:?}");
        let generator = Algorithm::from_str(subset.generator).unwrap();
        let checked = fill_table(htr_table, &mut subset_table, &generator, id - 1);
        total_checked += checked;
        debug!(
            "Checked {:width$}/{} cubes (new {})",
            total_checked,
            table_size,
            checked,
            width = table_size.to_string().len(),
        );
    }
    subset_table
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

fn fill_table(htr_table: &mut HTRPruningTable, subset_table: &mut HTRSubsetTable, generator: &Algorithm<Turn333>, subset_id: u8) -> usize {
    let mut total_checked = 0;
    let niss_bound = min_niss_moves(generator);
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
                    c.apply_alg(generator);
                    c
                })
        })
        .collect_vec();
    let mut check_next: Vec<Cube333> = vec![];
    loop {
        trace!("To check: {}", to_check.len());
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
                let (_, niss) = htr_table.get(coord);
                if niss != htr_table.empty_val() {
                    continue;
                }
                htr_table.set_niss(coord, niss_bound);
                subset_table.set(coord, subset_id);
                check_next.push(cube);
            }
        }
        if check_next.is_empty() {
            break total_checked;
        }
        total_checked += check_next.len();
        to_check = check_next;
        check_next = vec![];
    }
}