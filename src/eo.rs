use std::arch::x86_64::{__m128i, _mm_store_si128};
use std::cell::Ref;
use std::cmp::min;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;
use crate::algs::Algorithm;
use crate::alignment::AlignedU64;
use crate::coord::{Coord, EOCoord, EOCoordSingle};
use crate::cube::{Cube, FACES, Invertible, Move, Turn, Turnable, TURNS};
use crate::cube::Face::*;
use crate::cube::Turn::*;
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::df_search;
use crate::df_search::{dfs_iter};
use crate::lookup_table::Table;


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

pub fn eo_state_iter<C: Turnable + Invertible + EOCount + Clone + Copy>(cube: &C) -> impl Iterator<Item = Algorithm> + '_ {
    dfs_iter(UD_EO_STATE_CHANGE_MOVES, UD_EO_MOVES, Rc::new(|c: &C|{
        let (ud, fr, lr) = c.count_bad_edges();
        // let h = min(eo_heuristic(ud) + UD, eo_heuristic(fr) + FR);
        // let h: u8 = min(h, eo_heuristic(lr) + LR);
        // h as u32
        if ud == 0 { 0 } else { 1 }
    }), cube.clone(), true)
}

pub fn eo_state_iter_table<'a, C: Turnable + Invertible + Clone + Copy + 'a>(cube: &'a C, table: &'a Table<2048, EOCoordSingle>) -> impl Iterator<Item = Algorithm> + 'a
    where EOCoord: for<'x> From<&'x C> {
    let h = Rc::new(move |c: &C|{
        let EOCoord(udc, fbc, rlc) = EOCoord::from(c);
        // println!("{:?}", udc);
        let ud = table.get(udc).unwrap();
        // let fb = table.get(fbc).unwrap();
        // let rl = table.get(rlc).unwrap();
        // min(ud, min(fb, rl)) as u32
        ud as u32
    });
    dfs_iter(UD_EO_STATE_CHANGE_MOVES, UD_EO_MOVES, h, cube.clone(), true)
}

// pub fn eo_state_iter_table<'a, const UD: u8, const FR: u8, const LR: u8, C: Turnable + Invertible + Clone + Copy>(cube: &'a C, table: &'a Table<2047, &EOCoordSingle>) -> impl Iterator<Item = Algorithm> + 'a where EOCoord: for<'c> From<&'c C> {
//     dfs_iter(df_search::ALL_MOVES, &|c: &C|{
//         let EOCoord(udc, fbc, rlc) = EOCoord::from(c);
//         // let ud = table.get(&udc).unwrap();
//         // let fb = table.get(&fbc).unwrap();
//         // let rl = table.get(&rlc).unwrap();
//         //
//         // min(ud, min(fb, rl)) as u32
//     }, &cube, false)
// }

pub fn eo_heuristic(bad_edges: u8) -> u8 {
    (bad_edges + 2) / 4 + (bad_edges % 4) / 2
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