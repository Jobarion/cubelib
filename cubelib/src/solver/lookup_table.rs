use std::collections::HashMap;
use std::fmt::Debug;
#[cfg(feature = "fs")]
use std::fs;
#[cfg(feature = "fs")]
use std::fs::File;
use std::hash::Hash;
#[cfg(feature = "fs")]
use std::io::{Read, Write};
use std::marker::PhantomData;
#[cfg(feature = "fs")]
use home::home_dir;
use itertools::Itertools;
use log::{debug, info, warn};
use memmap2::Mmap;
use num_traits::{ToPrimitive};
#[cfg(feature = "fs")]
use num_traits::{FromPrimitive};
use crate::cube::*;
use crate::cube::turn::{Invertible, TurnableMut};
use crate::solver::moveset::MoveSet;
use crate::steps::coord::Coord;

const VERSION: u8 = 2;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum TableType {
    Uncompressed = 0u8,
    Compressed = 1u8,
    Niss = 2u8,
    Sym = 3u8,
}

pub struct MmapBinarySearchTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    pub(crate) coord_type: PhantomData<C>,
    pub entries: Mmap,
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> EmptyVal for MmapBinarySearchTable<C_SIZE, C> {
    fn empty_val(&self) -> u8 {
        0xFF
    }
}

#[derive(Clone)]
pub struct LookupTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8; C_SIZE]>,
    coord_type: PhantomData<C>,
    compressed: bool,
}

#[derive(Clone)]
pub struct NissLookupTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8]>,
    coord_type: PhantomData<C>,
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> MmapBinarySearchTable<C_SIZE, C> {
    pub fn get(&self, target: C) -> u8 {
        let mut size = self.entries.len() / 5;
        let mut base = 0;
        let target = target.val() as u32;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            let mid_idx = mid * 5;
            let c = u32::from_le_bytes(self.entries[mid_idx..(mid_idx+4)].try_into().unwrap());
            if c == target {
                return self.entries[mid_idx + 4]
            }
            base = if c < target {
                mid
            } else {
                base
            };
            size -= half;
        }
        let base_idx = base * 5;
        let c = u32::from_le_bytes(self.entries[base_idx..(base_idx+4)].try_into().unwrap());
        if c == target {
            self.entries[base_idx + 4]
        } else {
            0xFF
        }
    }
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> Into<Vec<u8>> for &LookupTable<C_SIZE, C> {
    fn into(self) -> Vec<u8> {
        let table_type = if self.compressed {
            TableType::Compressed
        } else {
            TableType::Uncompressed
        };
        let mut ser = vec![VERSION, table_type.to_u8().unwrap()];
        ser.extend(self.entries.iter());
        ser
    }
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> Into<Vec<u8>> for &NissLookupTable<C_SIZE, C> {
    fn into(self) -> Vec<u8> {
        let mut ser = vec![VERSION, TableType::Niss.to_u8().unwrap()];
        ser.extend(self.entries.iter());
        ser
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> LookupTable<C_SIZE, C> {
    pub fn new(compressed: bool) -> Self {
        let entries = if compressed {
            vec![0xFF; (C_SIZE + 1) / 2].into_boxed_slice().try_into().unwrap()
        } else {
            vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap()
        };
        LookupTable {
            entries,
            coord_type: PhantomData,
            compressed
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let table_type = if self.compressed {
            TableType::Compressed
        } else {
            TableType::Uncompressed
        };
        let mut ser = vec![VERSION, table_type.to_u8().unwrap()];
        ser.extend(self.entries.iter());
        ser
    }

    pub fn get(&self, id: C) -> u8 {
        let id: usize = id.into();
        if self.compressed {
            let entry = self.entries[id >> 1];
            let val = entry >> ((id & 1) << 2); //branchless entry << (id & 1 == 0 ? 0 : 4)
            val & 0x0F
        } else {
            self.entries[id]
        }
    }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        if self.compressed {
            let value = self.entries[id >> 1];
            let mask = 0xF0u8 >> ((id & 1) << 2);
            let entry = entry << ((id & 1) << 2);
            self.entries[id >> 1] = value & mask | entry;
        } else {
            self.entries[id] = entry
        }
    }

    #[cfg(feature = "fs")]
    pub fn load_and_save<F: FnMut() -> LookupTable<C_SIZE, C>>(key: &str, mut gen_f: F) -> (Self, bool) {
        match Self::load_from_disk("333", key) {
            Ok(t) => {
                debug!("Loaded {key} table from disk");
                (t, false)
            },
            Err(_) => {
                info!("Generating {key} table...");
                let table = gen_f();
                if let Err(e) = table.save_to_disk("333", key) {
                    warn!("Failed to save {key} table. {e}");
                }
                (table, true)
            }
        }
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> NissLookupTable<C_SIZE, C> {
    pub fn new() -> Self {
        NissLookupTable {
            entries: vec![0xFF; C_SIZE].into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
        }
    }

    pub fn empty_val(&self) -> u8 {
        0x0F
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut ser = vec![VERSION, TableType::Niss.to_u8().unwrap()];
        ser.extend(self.entries.iter());
        ser
    }

    pub fn get(&self, id: C) -> (u8, u8) {
        let id: usize = id.into();
        let entry = self.entries[id];
        (entry & 0x0F, entry >> 4)
    }

    pub fn set(&mut self, id: C, entry: u8) {
        let id: usize = id.into();
        self.entries[id] = (self.entries[id] & 0xF0) | (entry & 0x0F)
    }

    pub fn set_niss(&mut self, id: C, niss: u8) {
        let id: usize = id.into();
        self.entries[id] = (self.entries[id] & 0x0F) | (niss << 4)
    }

    #[cfg(feature = "fs")]
    pub fn load_and_save<F: FnMut() -> NissLookupTable<C_SIZE, C>>(key: &str, mut gen_f: F) -> Self {
        match Self::load_from_disk("333", key) {
            Ok(t) => {
                debug!("Loaded {key} table from disk");
                t
            },
            Err(_) => {
                info!("Generating {key} table...");
                let table = gen_f();
                if let Err(e) = table.save_to_disk("333", key) {
                    warn!("Failed to save {key} table. {e}");
                }
                table
            }
        }
    }
}

pub trait EmptyVal {
    fn empty_val(&self) -> u8;
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> EmptyVal for LookupTable<C_SIZE, C> {
    fn empty_val(&self) -> u8 {
        if self.compressed {
            0x0F
        } else {
            0xFF
        }
    }
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> EmptyVal for NissLookupTable<C_SIZE, C> {
    fn empty_val(&self) -> u8 {
        0x0F
    }
}

#[cfg(feature = "fs")]
pub trait SaveToDisk {
    fn save_to_disk(&self, puzzle_id: &str, table_type: &str) -> std::io::Result<()>;
}

#[cfg(feature = "fs")]
pub trait LoadFromDisk {
    fn load(data: Box<Vec<u8>>) -> Result<Self, String> where Self: Sized;

    fn load_from_disk(puzzle_id: &str, table_type: &str) -> Result<Self, String> where Self: Sized {
        let mut dir = home_dir().unwrap();
        dir.push(".cubelib");
        dir.push("tables");
        dir.push(puzzle_id);
        dir.push(format!("{table_type}.tbl"));
        debug!("Loading {puzzle_id} {table_type} table from {dir:?}");
        let mut file = File::open(dir).map_err(|e|e.to_string())?;
        let mut buffer = Box::new(Vec::new());
        file.read_to_end(&mut buffer).map_err(|e|e.to_string())?;
        Self::load(buffer)
    }
}

#[cfg(feature = "fs")]
impl <const C_SIZE: usize, C: Coord<C_SIZE>> LoadFromDisk for LookupTable<C_SIZE, C> {
    fn load(mut data: Box<Vec<u8>>) -> Result<Self, String> {
        let version = data[0];
        if version != VERSION {
            return Err("Invalid version".to_string())
        }
        let table_type: TableType = TableType::from_u8(data[1]).unwrap();
        assert_ne!(table_type, TableType::Niss);
        data.drain(0..2);
        if table_type == TableType::Compressed {
            assert_eq!(data.len(), (C_SIZE + 1) / 2);
        } else {
            assert_eq!(data.len(), C_SIZE);
        }

        Ok(LookupTable {
            entries: data.into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
            compressed: table_type == TableType::Compressed
        })
    }
}

#[cfg(feature = "fs")]
impl <const C_SIZE: usize, C: Coord<C_SIZE>> LoadFromDisk for NissLookupTable<C_SIZE, C> {
    fn load(mut data: Box<Vec<u8>>) -> Result<Self, String> {
        let version = data[0];
        if version != VERSION {
            return Err("Invalid version".to_string())
        }
        let table_type: TableType = TableType::from_u8(data[1]).unwrap();
        assert_eq!(table_type, TableType::Niss);
        data.drain(0..2);
        assert_eq!(data.len(), C_SIZE);

        Ok(NissLookupTable {
            entries: data.into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
        })
    }
}

#[cfg(feature = "fs")]
impl <const C_SIZE: usize, C: Coord<C_SIZE>> LoadFromDisk for MmapBinarySearchTable<C_SIZE, C> {
    fn load_from_disk(puzzle_id: &str, table_type: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        let mut dir = home_dir().unwrap();
        dir.push(".cubelib");
        dir.push("tables");
        dir.push(puzzle_id);
        dir.push(format!("{table_type}.tbl"));
        debug!("Loading {puzzle_id} {table_type} table from {dir:?}");
        let file = File::open(dir).map_err(|e|e.to_string())?;
        let mmap = unsafe {
            Mmap::map(&file)
        }.map_err(|e|e.to_string())?;
        Ok(Self {
            entries: mmap,
            coord_type: Default::default()
        })
    }

    fn load(_: Box<Vec<u8>>) -> Result<Self, String> {
        unimplemented!()
    }
}

#[cfg(feature = "fs")]
impl <T> SaveToDisk for T where for<'a> &'a T: Into<Vec<u8>> {
    fn save_to_disk(&self, puzzle_id: &str, table_type: &str) -> std::io::Result<()> {
        let mut dir = home_dir().unwrap();
        dir.push(".cubelib");
        dir.push("tables");
        dir.push(puzzle_id);
        fs::create_dir_all(dir.clone())?;
        dir.push(format!("{table_type}.tbl"));
        let mut file = File::create(dir)?;
        file.write_all(Into::<Vec<u8>>::into(self).as_slice())?;
        Ok(())
    }
}

pub fn generate<
    const COORD_SIZE: usize,
    Mapper,
    Table: EmptyVal,
    Init,
    Getter,
    Setter,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet,
    mapper: &Mapper,
    init: &Init,
    getter: &Getter,
    setter: &Setter,
) -> Table
where
    Mapper: Fn(&Cube333) -> CoordParam,
    Init: Fn() -> Table,
    Setter: Fn(&mut Table, CoordParam, u8),
    Getter: Fn(&Table, CoordParam) -> u8
{
    let start = Cube333::default();
    let mut visited = HashMap::new();
    let mut to_check = vec![start.clone()];
    visited.insert(mapper(&start), start);
    while !to_check.is_empty() {
        to_check = pre_gen_coset_0(&move_set, mapper, &mut visited, &to_check);
    }

    let mut to_check = HashMap::new();
    let mut table = init();
    for (start_coord, start_cube) in visited {
        setter(&mut table, start_coord, 0);
        to_check.insert(start_coord, start_cube);
    }
    if to_check.len() > 1 {
        debug!("Found {} variations of the goal state", to_check.len());
    }
    let mut total_checked = 0;
    for depth in 0.. {
        total_checked += to_check.len();
        debug!(
            "Checked {:width$}/{} cubes at depth {depth} (new {})",
            total_checked,
            CoordParam::size(),
            to_check.len(),
            width = CoordParam::size().to_string().len(),
        );
        to_check = fill_table(&move_set, &mut table, depth, mapper, getter, setter, to_check);
        if to_check.is_empty() {
            break;
        }
    }
    total_checked += to_check.len();
    if total_checked != CoordParam::size() {
        warn!(
            "Expected {} cubes in table but got {total_checked}. The coordinate may be malformed",
            CoordParam::size()
        );
    }
    table
}

fn pre_gen_coset_0<
    const COORD_SIZE: usize,
    Mapper,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet,
    mapper: &Mapper,
    visited: &mut HashMap<CoordParam, Cube333>,
    to_check: &Vec<Cube333>,
) -> Vec<Cube333>
where
    Mapper: Fn(&Cube333) -> CoordParam,
{
    let mut check_next = vec![];
    for cube in to_check {
        for m in move_set.aux_moves.iter().cloned() {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            if visited.contains_key(&coord) {
                continue;
            }
            visited.insert(coord, cube.clone());
            check_next.push(cube);
        }
    }
    check_next
}

fn fill_table<
    const COORD_SIZE: usize,
    Mapper,
    Table: EmptyVal,
    Getter,
    Setter,
    CoordParam: Coord<COORD_SIZE> + Copy + Hash + Eq + Debug,
>(
    move_set: &MoveSet,
    table: &mut Table,
    depth: u8,
    mapper: &Mapper,
    getter: &Getter,
    setter: &Setter,
    to_check: HashMap<CoordParam, Cube333>,
) -> HashMap<CoordParam, Cube333>
where
    Mapper: Fn(&Cube333) -> CoordParam,
    Setter: Fn(&mut Table, CoordParam, u8),
    Getter: Fn(&Table, CoordParam) -> u8
{
    let mut next_cubes: HashMap<CoordParam, Cube333> = HashMap::new();
    for (_coord, cube) in to_check.into_iter() {
        for m in move_set
            .aux_moves
            .into_iter()
            .chain(move_set.st_moves.into_iter())
            .cloned()
        {
            let mut cube = cube.clone();
            cube.turn(m);
            let coord = mapper(&cube);
            let stored = getter(&table, coord);
            if stored == table.empty_val() {
                setter(table, coord, depth + 1);
                next_cubes.insert(coord, cube);
            }
        }
    }
    next_cubes
}

// // This table is much larger and requires different methods to generate.
// // Since it's the only one we'll avoid a generic implementation for now
// pub fn generate_dr_finish_table<
//     const COORD_SIZE: usize,
//     CoordParam: Coord<COORD_SIZE> + for<'a> From<&'a Cube333> + From<usize> + Copy + Hash + Eq + Debug>(
//     symmetries: Vec<Symmetry>, move_set: MoveSet, mut file: File) -> std::io::Result<MmapBinarySearchTable<COORD_SIZE, CoordParam>> where for<'a> &'a CoordParam: Into<Cube333>  {
//
//     let mut hash_table: HashMap<CoordParam, u8> = HashMap::new();
//     let mut to_check = std::collections::HashSet::from([CoordParam::from(0)]);
//     let mut depth = 0;
//     while !to_check.is_empty() {
//         to_check.retain(|coord| hash_table.get(coord).is_none());
//         debug!("To check {} at depth {}", to_check.len(), depth);
//         let mut to_check_next = std::collections::HashSet::default();
//         for coord in to_check {
//             let mut cube = Into::<Cube333>::into(&coord);
//             hash_table.insert(coord, depth);
//             for turn in move_set.st_moves.iter().chain(move_set.aux_moves.iter()) {
//                 cube.turn(turn.clone());
//                 let coord = CoordParam::min_with_symmetries(&cube, &symmetries);
//                 if hash_table.get(&coord).is_none() {
//                     to_check_next.insert(coord);
//                 }
//                 cube.turn(turn.invert());
//             }
//         }
//         to_check = to_check_next;
//         depth += 1;
//     }
//     debug!("Collecting table");
//     let mut ordered_tuples = hash_table.into_iter().collect_vec();
//     debug!("Sorting");
//     ordered_tuples.sort_by(|(a, _), (b, _)|a.val().cmp(&b.val()));
//     debug!("Saving table");
//     for idx in 0..ordered_tuples.len() {
//         if idx % 1000 == 0 {
//             println!("{idx}");
//         }
//         let (c, d) = ordered_tuples[idx];
//         file.write_all((c.val() as u32).to_le_bytes().as_slice())?;
//         file.write_all(&[d])?;
//     }
//     Ok(MmapBinarySearchTable {
//         coord_type: Default::default(),
//         entries: unsafe { Mmap::map(&file) }?,
//     })
// }