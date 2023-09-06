use std::cmp::{max, min};
use std::str::FromStr;
use crate::algs::Algorithm;
use crate::cube::{Cube, Face, FACES, Move, Turn, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::CubieCube;
use crate::df_search;
use crate::df_search::{ALL_MOVES, dfs_iter};
use crate::moveset::{MoveSet, TransitionTable};

pub const UD_EO_FB_DR_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Right, Clockwise), Move(Right, CounterClockwise),
    Move(Left, Clockwise), Move(Left, CounterClockwise),
];

pub const UD_EO_FB_DR_MOVES: [Move; 10] = [
    Move(Up, Half),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Clockwise), Move(Front, CounterClockwise), Move(Front, Half),
    Move(Back, Clockwise), Move(Back, CounterClockwise), Move(Back, Half),
];

pub const UD_EO_FB_DR_MOVESET: MoveSet<4 , 10> = MoveSet {
    st_moves: UD_EO_FB_DR_STATE_CHANGE_MOVES,
    aux_moves: UD_EO_FB_DR_MOVES,
    transitions: dr_transitions(Left)
};

const CO_MASK_UD: u64 = 0x0707070707070707;

// pub fn ud_dr(cube: &CubieCube) -> impl Iterator<Item = Algorithm> + '_ {
//     find_ud_dr(&cube)
// }

// pub fn find_ud_dr(cube: &CubieCube) -> impl Iterator<Item = Algorithm> + '_ {
//     dfs_iter(ALL_MOVES, &|c: &CubieCube|{
//         let co_heuristic = co_heuristic(c.get_corners_raw());
//         let (_, fr, lr) = c.count_bad_edges();
//         let eo_heuristic = max(eo_heuristic(fr), eo_heuristic(lr));
//         max(eo_heuristic as u32, co_heuristic)
//     }, &cube, false)
// }

// pub fn find_ud_eo_fb_dr(cube: &CubieCube) -> impl Iterator<Item = Algorithm> + '_ {
//     dfs_iter(UD_EO_MOVES, &|c: &CubieCube|{
//         let co_heuristic = co_heuristic(c.get_corners_raw());
//         let (_, fr, lr) = c.count_bad_edges();
//         let eo_heuristic = max(eo_heuristic(fr), eo_heuristic(lr));
//         max(eo_heuristic as u32, co_heuristic)
//     }, &cube, true)
// }

pub fn co_heuristic(co: u64) -> u32 {
    let co_1_corners = (co & 0x0101010101010101).count_ones();
    let co_2_corners = (co & 0x0202020202020202).count_ones();
    let bad_corners = co_1_corners + co_2_corners;

    let co_estimate = (bad_corners + 3) / 4;
    co_estimate
}

const fn dr_transitions(dr_eo_axis: Face) -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    while i < FACES.len() {
        transitions[Move(FACES[i], Clockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], CounterClockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        i += 1;
    }
    i = 0;
    transitions[Move(axis_faces[0], Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_faces[0] as usize], TransitionTable::except_moves_to_mask([Move(axis_faces[1], Clockwise), Move(axis_faces[1], CounterClockwise)]));
    transitions[Move(axis_faces[1], Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_faces[1] as usize], TransitionTable::except_moves_to_mask([Move(axis_faces[0], Clockwise), Move(axis_faces[0], CounterClockwise)]));
    transitions
}