use std::cmp::{max, Ordering};
use std::cmp::Ordering::{Greater, Less};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::hint::black_box;
use std::io::repeat;
use std::iter::Map;
use std::marker::PhantomData;
use std::rc::Rc;
use itertools::{all, Itertools};
use num_enum::{IntoPrimitive, UnsafeFromPrimitive};
use crate::alignment::C;
use crate::coord::Coord;
use crate::cube::{Cube, Invertible, NewSolved};
use crate::df_search::ALL_MOVES;
use crate::{Algorithm, dfs_iter, Move, Turnable};
use crate::moveset::MoveSet;

pub struct Table<const CSize: usize, C: Coord<CSize>> {
    entries: Box<[u8; CSize]>,
    coord_type: PhantomData<C>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct MovesMod3(u8);

impl Ord for MovesMod3 {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (MovesMod3(0), MovesMod3(1)) => Less,
            (MovesMod3(1), MovesMod3(2)) => Less,
            (MovesMod3(2), MovesMod3(0)) => Less,
            (Unset, _) => Ordering::Less,
            (l, r) if l == r => Ordering::Equal,
            _ => Greater
        }
    }
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> Table<C_SIZE, C> {

    pub fn new() -> Self {
        Table {
            entries: vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData
        }
    }

    // pub fn get(&self, id: C) -> Option<MovesMod3> {
    //     let id = id.into();
    //     let entry = self.entries[id >> 2];
    //     let selected_bits: u8 = (entry >> ((id & 0b11) << 1)) & 0b11;
    //     match selected_bits {
    //         3 => None,
    //         n => Some(MovesMod3(n))
    //     }
    // }

    pub fn get(&self, id: C) -> Option<u8> {
        let id: usize = id.into();
        let entry = self.entries[id];
        match entry {
            0xFF => None,
            n => Some(n)
        }
    }

    // pub fn set(&mut self, id: C, entry: MovesMod3) {
    //     let id = id.into();
    //     let shift = (id & 0b11) << 1;
    //     let entry_bits = entry.0 << shift;
    //     let mask: u8 = !(0b11 << shift);
    //     self.entries[id >> 2] &= mask;
    //     self.entries[id >> 2] |= entry_bits;
    // }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        self.entries[id] = entry;
    }
}

pub fn dfs_table_heuristic<'a, const SC_SIZE: usize, const AUX_SIZE: usize, const COORD_SIZE: usize, CoordParam: Coord<COORD_SIZE>, C: Turnable + Invertible + Clone + Copy + 'a>(
    move_set: &'a MoveSet<SC_SIZE, AUX_SIZE>,
    table: &'a Table<COORD_SIZE, CoordParam>,
    cube: C,
    min_moves: u8,
    max_moves: u8,
    allow_niss: bool) -> impl Iterator<Item = Algorithm> + 'a
where CoordParam: for<'x> From<&'x C>
{
    let h = Rc::new(move |c: &C|{
        let coord = CoordParam::from(c);
        let heuristic = table.get(coord).unwrap();
        heuristic
    });
    dfs_iter(move_set, h, cube, min_moves, max_moves, allow_niss)
}

pub fn generate<const COORD_SIZE: usize, Mapper, CubeParam: Turnable + NewSolved + Clone, CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq, const ST_MOVES: usize, const AUX_MOVES: usize>(move_set: &MoveSet<ST_MOVES, AUX_MOVES>, mapper: &Mapper) -> Table<COORD_SIZE, CoordParam> where Mapper: Fn(&CubeParam) -> CoordParam {
    let mut table = Table::new();
    let mut to_check: HashMap<CoordParam, CubeParam> = HashMap::new();
    let start_cube = CubeParam::new_solved();
    let start_coord = mapper(&start_cube);
    table.set(start_coord, 0);
    to_check.insert(start_coord, start_cube);
    for depth in 0..20 {
        println!("Depth {} with {} cubes to check", depth, to_check.len());
        to_check = fill_table(move_set, &mut table, depth, &mapper, to_check);
        if to_check.is_empty() {
            break;
        }
    }
    table
}

fn fill_table<const COORD_SIZE: usize, Mapper, CubeParam: Turnable + NewSolved + Clone, CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq, const ST_MOVES: usize, const AUX_MOVES: usize>(move_set: &MoveSet<ST_MOVES, AUX_MOVES>, mut table: &mut Table<COORD_SIZE, CoordParam>, depth: u8, mapper: &Mapper, mut to_check: HashMap<CoordParam, CubeParam>) -> HashMap<CoordParam, CubeParam> where Mapper: Fn(&CubeParam) -> CoordParam {
    let mut next_cubes: HashMap<CoordParam, CubeParam> = HashMap::new();
    for (coord, cube) in to_check.into_iter() {
        for m in move_set.aux_moves {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            let stored = table.get(coord);
            if stored == None {
                table.set(coord, depth + 1);
                next_cubes.insert(coord, cube);
            }
        }
        for m in move_set.st_moves {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            let stored = table.get(coord);
            if stored == None {
                table.set(coord, depth + 1);
                next_cubes.insert(coord, cube);
            }
        }
    }
    next_cubes
}