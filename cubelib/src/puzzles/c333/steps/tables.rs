#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use log::{info, debug};
#[cfg(feature = "333dr")]
use crate::puzzles::c333::steps::dr::coords::DRUDEOFBCoord;
#[cfg(feature = "333eo")]
use crate::puzzles::c333::steps::eo::coords::EOCoordFB;
#[cfg(feature = "333finish")]
use crate::puzzles::c333::steps::finish::coords::FRUDFinishCoord;
#[cfg(feature = "333fr")]
use crate::puzzles::c333::steps::fr::coords::{FRUDNoSliceCoord, FRUDWithSliceCoord};
#[cfg(feature = "333htr")]
use crate::puzzles::c333::steps::htr::coords::HTRDRUDCoord;
use crate::solver::lookup_table;
use crate::solver::lookup_table::PruningTable;
#[cfg(feature = "333dr")]
use crate::puzzles::c333::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable};
#[cfg(feature = "333eo")]
use crate::puzzles::c333::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};
#[cfg(feature = "333finish")]
use crate::puzzles::c333::steps::finish::finish_config::{FRFinishPruningTable, FRUD_FINISH_MOVESET};
#[cfg(feature = "333fr")]
use crate::puzzles::c333::steps::fr::fr_config::{FR_UD_MOVESET, FRLeaveSlicePruningTable, FRPruningTable};
#[cfg(feature = "333htr")]
use crate::puzzles::c333::steps::htr::htr_config::{HTR_DR_UD_MOVESET, HTRPruningTable};

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
    fr_finish: Option<FRFinishPruningTable>
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
            fr_finish: None
        }
    }

    pub fn load(&mut self, key: &str, data: Vec<u8>) {
        match key {
            #[cfg(feature = "333eo")]
            "eo" => self.eo = Some(EOPruningTable::load(data)),
            #[cfg(feature = "333dr")]
            "dr" => self.dr = Some(DRPruningTable::load(data)),
            #[cfg(feature = "333htr")]
            "htr" => self.htr = Some(HTRPruningTable::load(data)),
            #[cfg(feature = "333fr")]
            "fr" => self.fr = Some(FRPruningTable::load(data)),
            #[cfg(feature = "333fr")]
            "frls" => self.frls = Some(FRLeaveSlicePruningTable::load(data)),
            #[cfg(feature = "333finish")]
            "frfin" => self.fr_finish = Some(FRFinishPruningTable::load(data)),
            _ => {}
        }
    }

    #[cfg(feature = "333eo")]
    pub fn gen_eo(&mut self) {
        if self.eo.is_none() {
            let table = gen_eo();
            self.eo = Some(table);
        }
    }

    #[cfg(feature = "333eo")]
    pub fn eo(&self) -> Option<&EOPruningTable> {
        self.eo.as_ref()
    }

    #[cfg(feature = "333dr")]
    pub fn gen_dr(&mut self) {
        if self.dr.is_none() {
            let table = gen_dr();
            self.dr = Some(table);
        }
    }

    #[cfg(feature = "333dr")]
    pub fn dr(&self) -> Option<&DRPruningTable> {
        self.dr.as_ref()
    }

    #[cfg(feature = "333htr")]
    pub fn gen_htr(&mut self) {
        if self.htr.is_none() {
            let table = gen_htr();
            self.htr = Some(table);
        }
    }

    #[cfg(feature = "333htr")]
    pub fn htr(&self) -> Option<&HTRPruningTable> {
        self.htr.as_ref()
    }

    #[cfg(feature = "333fr")]
    pub fn gen_fr_leave_slice(&mut self) {
        if self.frls.is_none() {
            let table = gen_fr_leave_slice();
            self.frls = Some(table);
        }
    }

    #[cfg(feature = "333fr")]
    pub fn fr_leave_slice(&self) -> Option<&FRLeaveSlicePruningTable> {
        self.frls.as_ref()
    }

    #[cfg(feature = "333fr")]
    pub fn gen_fr(&mut self) {
        if self.fr.is_none() {
            let table = gen_fr();
            self.fr = Some(table);
        }
    }

    #[cfg(feature = "333fr")]
    pub fn fr(&self) -> Option<&FRPruningTable> {
        self.fr.as_ref()
    }

    #[cfg(feature = "333finish")]
    pub fn gen_fr_finish(&mut self) {
        if self.fr_finish.is_none() {
            let table = gen_fr_finish();
            self.fr_finish = Some(table);
        }
    }

    #[cfg(feature = "333finish")]
    pub fn fr_finish(&self) -> Option<&FRFinishPruningTable> {
        self.fr_finish.as_ref()
    }
}

#[cfg(feature = "333eo")]
fn gen_eo() -> PruningTable<2048, EOCoordFB> {
    info!("Generating EO pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&EO_FB_MOVESET, &|c: &crate::puzzles::c333::EdgeCube333| EOCoordFB::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333dr")]
fn gen_dr() -> PruningTable<{ crate::puzzles::c333::steps::dr::coords::DRUDEOFB_SIZE }, DRUDEOFBCoord> {
    info!("Generating DR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET, &|c: &crate::puzzles::c333::Cube333| DRUDEOFBCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333htr")]
fn gen_htr() -> PruningTable<{ crate::puzzles::c333::steps::htr::coords::HTRDRUD_SIZE }, HTRDRUDCoord> {
    info!("Generating HTR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&HTR_DR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| HTRDRUDCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333fr")]
fn gen_fr_leave_slice() -> PruningTable<{ crate::puzzles::c333::steps::fr::coords::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord> {
    info!("Generating FRLS pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDNoSliceCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333fr")]
fn gen_fr() -> PruningTable<{ crate::puzzles::c333::steps::fr::coords::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord> {
    info!("Generating FR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDWithSliceCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "333finish")]
fn gen_fr_finish() -> PruningTable<{ crate::puzzles::c333::steps::finish::coords::FR_FINISH_SIZE }, FRUDFinishCoord> {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FRUD_FINISH_MOVESET, &|c: &crate::puzzles::c333::Cube333| FRUDFinishCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}