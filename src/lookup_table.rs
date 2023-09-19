use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::coord::Coord;
use crate::cube::{NewSolved, Turnable};
use crate::moveset::MoveSet;

pub struct PruningTable<const CSize: usize, C: Coord<CSize>> {
    entries: Box<[u8; CSize]>,
    coord_type: PhantomData<C>
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> PruningTable<C_SIZE, C> {

    pub fn new() -> Self {
        PruningTable {
            entries: vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData
        }
    }

    pub fn get(&self, id: C) -> Option<u8> {
        let id: usize = id.into();
        let entry = self.entries[id];
        match entry {
            0xFF => None,
            n => Some(n)
        }
    }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        self.entries[id] = entry;
    }
}

// pub fn dfs_table_heuristic<'a, const SC_SIZE: usize, const AUX_SIZE: usize, const PRE_TRANS: usize, const COORD_SIZE: usize, CoordParam: Coord<COORD_SIZE>, C: Turnable + Invertible + Clone + Copy + 'a>(
//     stage:
//     search_opts: SearchOptions,
//     table: &'a PruningTable<COORD_SIZE, CoordParam>,
//     cube: C) -> impl Iterator<Item = Algorithm> + 'a
// where CoordParam: for<'x> From<&'x C>
// {
//     let h = Rc::new(move |c: &C|{
//         let coord = CoordParam::from(c);
//         let heuristic = table.get(coord).unwrap();
//         heuristic
//     });
//     dfs_iter(h, cube, search_opts)
// }

pub fn generate<const COORD_SIZE: usize, Mapper, CubeParam: Turnable + NewSolved + Clone, CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq, const ST_MOVES: usize, const AUX_MOVES: usize>(move_set: &MoveSet<ST_MOVES, AUX_MOVES>, mapper: &Mapper) -> PruningTable<COORD_SIZE, CoordParam> where Mapper: Fn(&CubeParam) -> CoordParam {
    let mut table = PruningTable::new();
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

fn fill_table<const COORD_SIZE: usize, Mapper, CubeParam: Turnable + NewSolved + Clone, CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq, const ST_MOVES: usize, const AUX_MOVES: usize>(move_set: &MoveSet<ST_MOVES, AUX_MOVES>, table: &mut PruningTable<COORD_SIZE, CoordParam>, depth: u8, mapper: &Mapper, to_check: HashMap<CoordParam, CubeParam>) -> HashMap<CoordParam, CubeParam> where Mapper: Fn(&CubeParam) -> CoordParam {
    let mut next_cubes: HashMap<CoordParam, CubeParam> = HashMap::new();
    for (_, cube) in to_check.into_iter() {
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