use std::cmp::{max, min};
use std::rc::Rc;
use std::str::FromStr;
use crate::algs::Algorithm;
use crate::cube::{Cube, Face, FACES, Invertible, Move, Turn, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::CubieCube;
use crate::df_search::{ALL_MOVES, dfs_iter};
use crate::moveset::{MoveSet, TransitionTable};

pub const HTR_DR_UD_STATE_CHANGE_MOVES: [Move; 4] = [
    Move(Up, Clockwise), Move(Up, CounterClockwise),
    Move(Down, Clockwise), Move(Down, CounterClockwise),
];

pub const HTR_DR_UD_MOVES: [Move; 6] = [
    Move(Up, Half),
    Move(Down, Half),
    Move(Right, Half),
    Move(Left, Half),
    Move(Front, Half),
    Move(Back, Half),
];

pub const HTR_DR_UD_MOVESET: MoveSet<4 , 6> = MoveSet {
    st_moves: HTR_DR_UD_STATE_CHANGE_MOVES,
    aux_moves: HTR_DR_UD_MOVES,
    transitions: htr_transitions(Up)
};

const fn htr_transitions(axis_face: Face) -> [TransitionTable; 18] {
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