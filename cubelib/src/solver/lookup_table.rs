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
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};
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
use rayon::prelude::*;
use crate::solver::moveset::MoveSet;
use crate::steps::coord::Coord;
use crate::steps::MoveSet333;

const VERSION: u8 = 2;

pub trait DepthEstimate<const C_SIZE: usize, C: Coord<C_SIZE>>: Send + Sync {
    fn get(&self, target: C) -> u8;
}

pub trait NissDepthEstimate<const C_SIZE: usize, C: Coord<C_SIZE>>: Send + Sync {
    fn get_niss_estimate(&self, target: C) -> (u8, u8);
}

// impl <const C_SIZE: usize, C: Coord<C_SIZE>, NDE: NissDepthEstimate<C_SIZE, C>> DepthEstimate<C_SIZE, C> for NDE {
//     fn get(&self, target: C) -> u8 {
//         self.get_niss_estimate(target).0
//     }
// }

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum TableType {
    Uncompressed = 0u8,
    Compressed = 1u8,
    Niss = 2u8,
    Sym = 3u8,
}

pub struct HashTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: HashMap<usize, u8>,
    coord_type: PhantomData<C>,
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> HashTable<C_SIZE, C> {
    pub fn new() -> Self {
        Self {
            entries: Default::default(),
            coord_type: Default::default(),
        }
    }
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> EmptyVal for HashTable<C_SIZE, C> {
    fn empty_val(&self) -> u8 {
        0xFF
    }
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
pub struct ArrayTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    pub(crate) entries: Box<[u8]>,
    coord_type: PhantomData<C>,
    compressed: bool,
}

#[derive(Clone)]
pub struct NissLookupTable<const C_SIZE: usize, C: Coord<C_SIZE>> {
    entries: Box<[u8]>,
    coord_type: PhantomData<C>,
}

impl <const C_SIZE: usize, C: Coord<C_SIZE>> Into<Vec<u8>> for &ArrayTable<C_SIZE, C> {
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

impl<const C_SIZE: usize, C: Coord<C_SIZE>> DepthEstimate<C_SIZE, C> for HashTable<C_SIZE, C> {
    fn get(&self, id: C) -> u8 {
        self.entries.get(&id.val()).cloned().unwrap_or(self.empty_val())
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> DepthEstimate<C_SIZE, C> for ArrayTable<C_SIZE, C> {
    fn get(&self, id: C) -> u8 {
        self.get_direct(id)
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> DepthEstimate<C_SIZE, C> for MmapBinarySearchTable<C_SIZE, C> {
    fn get(&self, target: C) -> u8 {
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

impl<const C_SIZE: usize, C: Coord<C_SIZE>> NissDepthEstimate<C_SIZE, C> for NissLookupTable<C_SIZE, C> {
    fn get_niss_estimate(&self, target: C) -> (u8, u8) {
        let id: usize = target.into();
        let entry = self.entries[id];
        (entry & 0x0F, entry >> 4)
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> DepthEstimate<C_SIZE, C> for NissLookupTable<C_SIZE, C> {
    fn get(&self, id: C) -> u8 {
        self.get_niss_estimate(id).0
    }
}

impl<const C_SIZE: usize, C: Coord<C_SIZE>> ArrayTable<C_SIZE, C> {
    pub fn new(compressed: bool) -> Self {
        Self::new_with_size(compressed, C_SIZE)
    }

    pub fn new_with_size(compressed: bool, size: usize) -> Self {
        let size = size.min(C_SIZE);
        let entries = if compressed {
            vec![0xFF; (size + 1) / 2].into_boxed_slice()
        } else {
            vec![0xFF; size].into_boxed_slice()
        };
        ArrayTable {
            entries,
            coord_type: PhantomData,
            compressed,
        }
    }

    pub fn get_direct<A: Into<usize>>(&self, id: A) -> u8 {
        let id: usize = id.into();
        if self.compressed {
            let entry = self.entries[id >> 1];
            let val = entry >> ((id & 1) << 2);
            val & 0x0F
        } else {
            self.entries[id]
        }
    }

    pub fn set_direct<A: Into<usize>>(&mut self, id: A, entry: u8) {
        let id: usize = id.into();
        let entry = entry & 0xF;
        if self.compressed {
            let value = self.entries[id >> 1];
            let mask = 0xF0u8 >> ((id & 1) << 2);
            let entry = entry << ((id & 1) << 2);
            self.entries[id >> 1] = value & mask | entry;
        } else {
            self.entries[id] = entry;
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

    pub fn set(&mut self, id: C, entry: u8) {
        self.set_direct(id, entry)
    }

    #[cfg(feature = "fs")]
    pub fn load_and_save<F: FnMut() -> ArrayTable<C_SIZE, C>>(key: &str, mut gen_f: F) -> (Self, bool) {
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

impl <const C_SIZE: usize, C: Coord<C_SIZE>> EmptyVal for ArrayTable<C_SIZE, C> {
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
impl <const C_SIZE: usize, C: Coord<C_SIZE>> LoadFromDisk for ArrayTable<C_SIZE, C> {
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

        Ok(ArrayTable {
            entries: data.into_boxed_slice().try_into().unwrap(),
            coord_type: PhantomData,
            compressed: table_type == TableType::Compressed,
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

// This table is much larger and requires different methods to generate.
// Since it's the only one we'll avoid a generic implementation for now
pub fn generate_large_table<
    const C_SIZE: usize,
    C: Coord<C_SIZE>,
>(move_set: &MoveSet333) -> ArrayTable<{C_SIZE}, C> where for <'a> C: From<&'a Cube333>, for <'a> &'a C: Into<Cube333>, C: From<usize> {
    let mut table = ArrayTable::new(true);
    table.set_direct(C::from(&Cube333::default()), 0);

    let mut depth = 1;
    let empty_val = table.empty_val();
    let mut table = table;
    while depth < empty_val {
        let mut changed = 0usize;
        let mut touched = 0usize;
        let mut last_percentage = 0u8;
        (0..C::size())
            .into_iter()
            .for_each(|idx|{
                touched += 1;
                let percentage = (touched * 10 / C::size()) as u8;
                if percentage - 1 == last_percentage {
                    debug!("Depth {depth}/{}, {}%", empty_val - 1, percentage * 10);
                    last_percentage = percentage;
                }
                if table.get_direct(idx) == depth - 1 {
                    let mut cube: Cube333 = (&C::from(idx)).into();
                    for turn in move_set.st_moves.iter().chain(move_set.aux_moves.iter()) {
                        cube.turn(turn.clone());
                        let coord = C::from(&cube);
                        if table.get(coord) == empty_val {
                            changed += 1;
                            table.set(coord, depth);
                        }
                        cube.turn(turn.invert());
                    }
                }
            });
        // for idx in 0..C::size() {
        //     if last_msg.elapsed() > Duration::from_secs(60) {
        //         debug!("Depth {depth}, {}%", idx * 100 / C::size());
        //         last_msg = Instant::now();
        //     }
        //     if table.get_direct(idx) == depth - 1 {
        //         let mut cube: Cube333 = (&C::from(idx)).into();
        //         for turn in move_set.st_moves.iter().chain(move_set.aux_moves.iter()) {
        //             cube.turn(turn.clone());
        //             let coord = C::from(&cube);
        //             if table.get(coord) == table.empty_val() {
        //                 changed += 1;
        //                 table.set(coord, depth);
        //             }
        //             cube.turn(turn.invert());
        //         }
        //     }
        // }
        debug!("Positions at depth {depth}: {}", changed);
        depth += 1;
    }
    table
}
