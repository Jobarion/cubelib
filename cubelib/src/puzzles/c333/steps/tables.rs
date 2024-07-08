#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use log::{debug, error, info};

#[cfg(feature = "333dr")]
use crate::puzzles::c333::steps::dr::coords::DRUDEOFBCoord;
#[cfg(feature = "333dr")]
use crate::puzzles::c333::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable};
#[cfg(feature = "333eo")]
use crate::puzzles::c333::steps::eo::coords::EOCoordFB;
#[cfg(feature = "333eo")]
use crate::puzzles::c333::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};
#[cfg(feature = "333finish")]
use crate::puzzles::c333::steps::finish::coords::FRUDFinishCoord;
use crate::puzzles::c333::steps::finish::coords::HTRFinishCoord;
#[cfg(feature = "333finish")]
use crate::puzzles::c333::steps::finish::finish_config::{FRFinishPruningTable, FRUD_FINISH_MOVESET};
use crate::puzzles::c333::steps::finish::finish_config::{HTR_FINISH_MOVESET, HTRFinishPruningTable};
#[cfg(feature = "333fr")]
use crate::puzzles::c333::steps::fr::coords::{FRUDNoSliceCoord, FRUDWithSliceCoord};
#[cfg(feature = "333fr")]
use crate::puzzles::c333::steps::fr::fr_config::{FR_UD_MOVESET, FRLeaveSlicePruningTable, FRPruningTable};
use crate::puzzles::c333::steps::htr;
#[cfg(feature = "333htr")]
use crate::puzzles::c333::steps::htr::coords::HTRDRUDCoord;
#[cfg(feature = "333htr")]
use crate::puzzles::c333::steps::htr::htr_config::{HTR_DR_UD_MOVESET, HTRPruningTable};
use crate::solver::lookup_table;
use crate::solver::lookup_table::{PruningTable, TableType};
use crate::steps::coord::Coord;

#[derive(Clone)]
pub struct PruningTables333 {
    #[cfg(feature = "333eo")]
    eo: Option<EOPruningTable>,
    #[cfg(feature = "333dr")]
    dr: Option<DRPruningTable>,
    #[cfg(feature = "333htr")]
    htr: Option<HTRPruningTable>,
    #[cfg(feature = "333fr")]
    frls: Option<FRLeaveSlicePruningTable>,
    #[cfg(feature = "333fr")]
    fr: Option<FRPruningTable>,
    #[cfg(feature = "333finish")]
    fr_finish: Option<FRFinishPruningTable>,
    #[cfg(feature = "333finish")]
    htr_finish: Option<HTRFinishPruningTable>
}

impl PruningTables333 {

    pub const VERSION: u32 = 1;

    pub fn new() -> PruningTables333 {
        PruningTables333 {
            #[cfg(feature = "333eo")]
            eo: None,
            #[cfg(feature = "333dr")]
            dr: None,
            #[cfg(feature = "333htr")]
            htr: None,
            #[cfg(feature = "333fr")]
            frls: None,
            #[cfg(feature = "333fr")]
            fr: None,
            #[cfg(feature = "333finish")]
            fr_finish: None,
            #[cfg(feature = "333finish")]
            htr_finish: None
        }
    }

    pub fn save(&self, key: &str) -> std::io::Result<()> {
        match key {
            #[cfg(feature = "333eo")]
            "eo" => if let Some(tbl) = &self.eo {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333dr")]
            "dr" => if let Some(tbl) = &self.dr {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333htr")]
            "htr" => if let Some(tbl) = &self.htr {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333fr")]
            "fr" => if let Some(tbl) = &self.fr {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333fr")]
            "frls" => if let Some(tbl) = &self.frls {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333finish")]
            "frfin" => if let Some(tbl) = &self.fr_finish {
                tbl.save_to_disk("333", key)?
            },
            #[cfg(feature = "333finish")]
            "htrfin" => if let Some(tbl) = &self.htr_finish {
                tbl.save_to_disk("333", key)?
            },
            _ => {}
        }
        Ok(())
    }

    pub fn load(&mut self, key: &str) -> Result<(), String> {
        match key {
            #[cfg(feature = "333eo")]
            "eo" => self.eo = Some(EOPruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333dr")]
            "dr" => self.dr = Some(DRPruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333htr")]
            "htr" => self.htr = Some(HTRPruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333fr")]
            "fr" => self.fr = Some(FRPruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333fr")]
            "frls" => self.frls = Some(FRLeaveSlicePruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333finish")]
            "frfin" => self.fr_finish = Some(FRFinishPruningTable::load_from_disk("333", key)?),
            #[cfg(feature = "333finish")]
            "htrfin" => self.htr_finish = Some(HTRFinishPruningTable::load_from_disk("333", key)?),
            _ => {}
        }
        Ok(())
    }

    #[cfg(feature = "fs")]
    pub fn load_and_gen<const C_SIZE: usize, C: Coord<C_SIZE>>(key: &str, val: &mut Option<PruningTable<C_SIZE, C>>, gen_f: &dyn Fn() -> PruningTable<C_SIZE, C>, load_f: &dyn Fn() -> Result<PruningTable<C_SIZE, C>, String>) -> bool {
        if val.is_none() {
            let res = load_f();
            match res {
                Ok(v) => {
                    *val = Some(v);
                    info!("Loaded {key} table from disk");
                },
                Err(e) => {
                    error!("Error loading {key} table from disk: {e}");
                }
            }
        }
        if val.is_none() {
            let table = gen_f();
            *val = Some(table);
            return true;
        }
        false
    }

    #[cfg(feature = "fs")]
    pub fn load_and_save<const C_SIZE: usize, C: Coord<C_SIZE>>(&mut self, key: &str, mut_f: &dyn Fn(&mut Self) -> &mut Option<PruningTable<C_SIZE, C>>, gen_f: &dyn Fn() -> PruningTable<C_SIZE, C>, load_f: &dyn Fn() -> Result<PruningTable<C_SIZE, C>, String>) {
        let should_save = Self::load_and_gen(key, mut_f(self), gen_f, load_f);
        if should_save {
            if let Err(e) = self.save(key) {
                error!("Error saving {key} table to disk: {e}");
            } else {
                info!("Saved {key} table to disk");
            }
        }
    }

    #[cfg(all(feature = "333eo", feature = "fs"))]
    pub fn gen_eo(&mut self) {
        self.load_and_save("eo", &|x|&mut x.eo, &gen_eo, &|| EOPruningTable::load_from_disk("333", "eo"));
    }

    #[cfg(all(feature = "333eo", not(feature = "fs")))]
    pub fn gen_eo(&mut self) {
        self.eo = Some(gen_eo());
    }

    #[cfg(feature = "333eo")]
    pub fn eo(&self) -> Option<&EOPruningTable> {
        self.eo.as_ref()
    }

    #[cfg(all(feature = "333dr", feature = "fs"))]
    pub fn gen_dr(&mut self) {
        self.load_and_save("dr", &|x|&mut x.dr, &gen_dr, &|| DRPruningTable::load_from_disk("333", "dr"));
    }

    #[cfg(all(feature = "333dr", not(feature = "fs")))]
    pub fn gen_dr(&mut self) {
        self.dr = Some(gen_dr());
    }

    #[cfg(feature = "333dr")]
    pub fn dr(&self) -> Option<&DRPruningTable> {
        self.dr.as_ref()
    }

    #[cfg(all(feature = "333htr", feature = "fs"))]
    pub fn gen_htr(&mut self) {
        self.load_and_save("htr", &|x|&mut x.htr, &gen_htr, &|| HTRPruningTable::load_from_disk("333", "htr"));
    }

    #[cfg(all(feature = "333htr", not(feature = "fs")))]
    pub fn gen_htr(&mut self) {
        self.htr = Some(gen_htr());
    }

    #[cfg(feature = "333htr")]
    pub fn htr(&self) -> Option<&HTRPruningTable> {
        self.htr.as_ref()
    }

    #[cfg(feature = "333htr")]
    pub fn htr_mut(&mut self) -> Option<&mut HTRPruningTable> {
        self.htr.as_mut()
    }

    #[cfg(all(feature = "333fr", feature = "fs"))]
    pub fn gen_fr_leave_slice(&mut self) {
        self.load_and_save("frls", &|x|&mut x.frls, &gen_fr_leave_slice, &|| FRLeaveSlicePruningTable::load_from_disk("333", "frls"));
    }

    #[cfg(all(feature = "333fr", not(feature = "fs")))]
    pub fn gen_fr_leave_slice(&mut self) {
        self.frls = Some(gen_fr_leave_slice());
    }

    #[cfg(feature = "333fr")]
    pub fn fr_leave_slice(&self) -> Option<&FRLeaveSlicePruningTable> {
        self.frls.as_ref()
    }

    #[cfg(feature = "333fr")]
    pub fn gen_fr(&mut self) {
        self.load_and_save("fr", &|x|&mut x.fr, &gen_fr, &|| FRPruningTable::load_from_disk("333", "fr"));
    }

    #[cfg(all(feature = "333fr", not(feature = "fs")))]
    pub fn gen_fr(&mut self) {
        self.fr = Some(gen_fr());
    }

    #[cfg(feature = "333fr")]
    pub fn fr(&self) -> Option<&FRPruningTable> {
        self.fr.as_ref()
    }

    #[cfg(all(feature = "333finish", feature = "fs"))]
    pub fn gen_fr_finish(&mut self) {
        self.load_and_save("frfin", &|x|&mut x.fr_finish, &gen_fr_finish, &|| FRFinishPruningTable::load_from_disk("333", "frfin"));
    }

    #[cfg(all(feature = "333finish", not(feature = "fs")))]
    pub fn gen_fr_finish(&mut self) {
        self.fr_finish = Some(gen_fr_finish());
    }

    #[cfg(feature = "333finish")]
    pub fn fr_finish(&self) -> Option<&FRFinishPruningTable> {
        self.fr_finish.as_ref()
    }

    #[cfg(all(feature = "333finish", feature = "fs"))]
    pub fn gen_htr_finish(&mut self) {
        self.load_and_save("htrfin", &|x|&mut x.htr_finish, &gen_htr_finish, &|| HTRFinishPruningTable::load_from_disk("333", "htrfin"));
    }

    #[cfg(all(feature = "333finish", not(feature = "fs")))]
    pub fn gen_htr_finish(&mut self) {
        self.htr_finish = Some(crate::puzzles::c333::steps::tables::gen_htr_finish());
    }

    #[cfg(feature = "333finish")]
    pub fn htr_finish(&self) -> Option<&HTRFinishPruningTable> {
        self.htr_finish.as_ref()
    }
}

#[cfg(feature = "333eo")]
fn gen_eo() -> PruningTable<2048, EOCoordFB> {
    info!("Generating EO pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&EO_FB_MOVESET, &|c: &crate::puzzles::c333::EdgeCube333| EOCoordFB::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333dr")]
fn gen_dr() -> PruningTable<{ crate::puzzles::c333::steps::dr::coords::DRUDEOFB_SIZE }, DRUDEOFBCoord> {
    info!("Generating DR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET, &|c: &crate::puzzles::c333::Cube333| DRUDEOFBCoord::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333htr")]
fn gen_htr() -> PruningTable<{ crate::puzzles::c333::steps::htr::coords::HTRDRUD_SIZE }, HTRDRUDCoord> {
    info!("Generating HTR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let mut table = lookup_table::generate(&HTR_DR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| HTRDRUDCoord::from(c), TableType::Niss);
    htr::util::gen_niss_table(&mut table);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333fr")]
fn gen_fr_leave_slice() -> PruningTable<{ crate::puzzles::c333::steps::fr::coords::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord> {
    info!("Generating FRLS pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDNoSliceCoord::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333fr")]
fn gen_fr() -> PruningTable<{ crate::puzzles::c333::steps::fr::coords::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord> {
    info!("Generating FR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDWithSliceCoord::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333finish")]
fn gen_fr_finish() -> PruningTable<{ crate::puzzles::c333::steps::finish::coords::FR_FINISH_SIZE }, FRUDFinishCoord> {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FRUD_FINISH_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDFinishCoord::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333finish")]
fn gen_htr_finish() -> PruningTable<{ crate::puzzles::c333::steps::finish::coords::HTR_FINISH_SIZE }, HTRFinishCoord> {
    info!("Generating HTR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&HTR_FINISH_MOVESET, &|c: &crate::puzzles::c333::Cube333| HTRFinishCoord::from(c), TableType::Uncompressed);
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}