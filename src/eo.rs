use std::arch::x86_64::{__m128i, _mm_store_si128};
use std::cell::Ref;
use std::cmp::min;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;
use crate::algs::Algorithm;
use crate::alignment::AlignedU64;
use crate::coord::{Coord, EOCoordAll, EOCoordFB, EOCoordUD};
use crate::cube::{Cube, Face, FACES, Invertible, Move, Turn, Turnable, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search;
use crate::df_search::{dfs_iter};
use crate::lookup_table::Table;
use crate::moveset::{MoveSet, Transition, TransitionTable};


pub const UD_EO_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Up, Clockwise), Move(Up, CounterClockwise),
    Move(Down, Clockwise), Move(Down, CounterClockwise),
];

pub const UD_EO_MOVES: [Move; 14] = [
    Move(Up, Half),
    Move(Down, Half),
    Move(Front, Clockwise), Move(Front, CounterClockwise), Move(Front, Half),
    Move(Back, Clockwise), Move(Back, CounterClockwise), Move(Back, Half),
    Move(Left, Clockwise), Move(Left, CounterClockwise), Move(Left, Half),
    Move(Right, Clockwise), Move(Right, CounterClockwise), Move(Right, Half),
];

pub const FB_EO_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Front, Clockwise), Move(Front, CounterClockwise),
    Move(Back, Clockwise), Move(Back, CounterClockwise),
];

pub const FB_EO_MOVES: [Move; 14] = [
    Move(Up, Clockwise), Move(Up, CounterClockwise), Move(Up, Half),
    Move(Down, Clockwise), Move(Down, CounterClockwise), Move(Down, Half),
    Move(Front, Half),
    Move(Back, Half),
    Move(Left, Clockwise), Move(Left, CounterClockwise), Move(Left, Half),
    Move(Right, Clockwise), Move(Right, CounterClockwise), Move(Right, Half),
];

pub const RL_EO_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Right, Clockwise), Move(Left, CounterClockwise),
    Move(Right, Clockwise), Move(Left, CounterClockwise),
];

pub const RL_EO_MOVES: [Move; 14] = [
    Move(Up, Clockwise), Move(Up, CounterClockwise), Move(Up, Half),
    Move(Down, Clockwise), Move(Down, CounterClockwise), Move(Down, Half),
    Move(Front, Clockwise), Move(Front, CounterClockwise), Move(Front, Half),
    Move(Back, Clockwise), Move(Back, CounterClockwise), Move(Back, Half),
    Move(Left, Half),
    Move(Right, Half),
];

pub const EO_UD_MOVESET: MoveSet<4, 14> = MoveSet {
    st_moves: UD_EO_STATE_CHANGE_MOVES,
    aux_moves: UD_EO_MOVES,
    transitions: eo_transitions(Up)
};

pub const EO_FB_MOVESET: MoveSet<4, 14> = MoveSet {
    st_moves: FB_EO_STATE_CHANGE_MOVES,
    aux_moves: FB_EO_MOVES,
    transitions: eo_transitions(Front)
};

pub const EO_RL_MOVESET: MoveSet<4, 14> = MoveSet {
    st_moves: RL_EO_STATE_CHANGE_MOVES,
    aux_moves: RL_EO_MOVES,
    transitions: eo_transitions(Left)
};

// const BAD_EDGES_MIN_MOVES: [u8; 13] = [0, 99, 3, 99, 1, 99, 3, 99, 2, 99, 6, 99, 8];
//
// pub fn eo_ud_state_iter<'a, C: Turnable + Invertible + EOCount + Clone + Copy + 'a>(cube: C) -> impl Iterator<Item = Algorithm> + 'a {
//     dfs_iter(&EO_UD_MOVESET, Rc::new(|c: &C|{
//         let (ud, _, _) = c.count_bad_edges();
//         BAD_EDGES_MIN_MOVES[ud as usize]
//     }), cube, 0, 20, true)
// }

pub fn filter_eo_last_moves_pure(alg: &Algorithm) -> bool {
    filter_last_moves_pure(&alg.normal_moves) && filter_last_moves_pure(&alg.inverse_moves)
}

fn filter_last_moves_pure(vec: &Vec<Move>) -> bool {
    match vec.len() {
        0 => true,
        1 => vec[0].1 != CounterClockwise,
        n => {
            if vec[n - 1].1 == CounterClockwise {
                false
            } else {
                if vec[n - 1].0.opposite() == vec[n - 2].0 {
                    vec[n - 2].1 != CounterClockwise
                }
                else {
                    true
                }
            }
        } ,
    }
}

pub trait EOCount {
    fn count_bad_edges(&self) -> (u8, u8, u8);
}

impl EOCount for CubieCube {
    fn count_bad_edges(&self) -> (u8, u8, u8) {
        self.edges.count_bad_edges()
    }
}

impl EOCount for EdgeCubieCube {

    fn count_bad_edges(&self) -> (u8, u8, u8) {
        let edges = self.get_edges_raw();
        let ud = (edges[0] & CubieCube::BAD_EDGE_MASK_UD).count_ones() + (edges[1] & CubieCube::BAD_EDGE_MASK_UD).count_ones();
        let fb = (edges[0] & CubieCube::BAD_EDGE_MASK_FB).count_ones() + (edges[1] & CubieCube::BAD_EDGE_MASK_FB).count_ones();
        let rl = (edges[0] & CubieCube::BAD_EDGE_MASK_RL).count_ones() + (edges[1] & CubieCube::BAD_EDGE_MASK_RL).count_ones();
        (ud as u8, fb as u8, rl as u8)
    }
}

const fn eo_transitions(axis_face: Face) -> [TransitionTable; 18] {
    let mut transitions = [TransitionTable::new(0, 0); 18];
    let mut i = 0;
    while i < FACES.len() {
        transitions[Move(FACES[i], Clockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        transitions[Move(FACES[i], CounterClockwise).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[FACES[i] as usize], TransitionTable::ANY);
        i += 1;
    }
    i = 0;
    transitions[Move(axis_face, Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face as usize], TransitionTable::except_moves_to_mask([Move(axis_face.opposite(), Clockwise), Move(axis_face.opposite(), CounterClockwise)]));
    transitions[Move(axis_face.opposite(), Half).to_id()] = TransitionTable::new(TransitionTable::DEFAULT_ALLOWED_AFTER[axis_face.opposite() as usize], TransitionTable::except_moves_to_mask([Move(axis_face, Clockwise), Move(axis_face,CounterClockwise)]));
    transitions
}