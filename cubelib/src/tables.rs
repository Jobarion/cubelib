use std::time::Instant;
use log::{debug, info};
use crate::coords::{dr, finish, fr, htr};
use crate::coords::dr::DRUDEOFBCoord;
use crate::coords::eo::EOCoordFB;
use crate::coords::finish::FRUDFinishCoord;
use crate::coords::fr::{FRUDNoSliceCoord, FRUDWithSliceCoord};
use crate::coords::htr::HTRDRUDCoord;
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::lookup_table;
use crate::lookup_table::PruningTable;
use crate::steps::dr::{DR_UD_EO_FB_MOVESET, DRPruningTable};
use crate::steps::eo;
use crate::steps::eo::EOPruningTable;
use crate::steps::finish::{FRFinishPruningTable, FRUD_FINISH_MOVESET};
use crate::steps::fr::{FR_UD_MOVESET, FRLeaveSlicePruningTable, FRPruningTable};
use crate::steps::htr::{HTR_DR_UD_MOVESET, HTRPruningTable};

pub struct PruningTables {
    eo: Option<EOPruningTable>,
    dr: Option<DRPruningTable>,
    htr: Option<HTRPruningTable>,
    frls: Option<FRLeaveSlicePruningTable>,
    fr: Option<FRPruningTable>,
    fr_finish: Option<FRFinishPruningTable>
}


impl PruningTables {

    pub fn new() -> PruningTables {
        PruningTables {
            eo: None,
            dr: None,
            htr: None,
            frls: None,
            fr: None,
            fr_finish: None
        }
    }

    pub fn gen_eo(&mut self) {
        if self.eo.is_none() {
            let table = gen_eo();
            self.eo = Some(table);
        }
    }

    pub fn eo(&self) -> Option<&EOPruningTable> {
        self.eo.as_ref()
    }

    pub fn gen_dr(&mut self) {
        if self.dr.is_none() {
            let table = gen_dr();
            self.dr = Some(table);
        }
    }

    pub fn dr(&self) -> Option<&DRPruningTable> {
        self.dr.as_ref()
    }

    pub fn gen_htr(&mut self) {
        if self.htr.is_none() {
            let table = gen_htr();
            self.htr = Some(table);
        }
    }

    pub fn htr(&self) -> Option<&HTRPruningTable> {
        self.htr.as_ref()
    }

    pub fn gen_fr_leave_slice(&mut self) {
        if self.frls.is_none() {
            let table = gen_fr_leave_slice();
            self.frls = Some(table);
        }
    }

    pub fn fr_leave_slice(&self) -> Option<&FRLeaveSlicePruningTable> {
        self.frls.as_ref()
    }

    pub fn gen_fr(&mut self) {
        if self.fr.is_none() {
            let table = gen_fr();
            self.fr = Some(table);
        }
    }

    pub fn fr(&self) -> Option<&FRPruningTable> {
        self.fr.as_ref()
    }

    pub fn gen_fr_finish(&mut self) {
        if self.fr_finish.is_none() {
            let table = gen_fr_finish();
            self.fr_finish = Some(table);
        }
    }

    pub fn fr_finish(&self) -> Option<&FRFinishPruningTable> {
        self.fr_finish.as_ref()
    }
}

fn gen_eo() -> PruningTable<2048, EOCoordFB> {
    info!("Generating EO pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&eo::EO_FB_MOVESET, &|c: &EdgeCubieCube| EOCoordFB::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

fn gen_dr() -> PruningTable<{ dr::DRUDEOFB_SIZE }, DRUDEOFBCoord> {
    info!("Generating DR pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET, &|c: &CubieCube| DRUDEOFBCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

fn gen_htr() -> PruningTable<{ htr::HTRDRUD_SIZE }, HTRDRUDCoord> {
    info!("Generating HTR pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&HTR_DR_UD_MOVESET, &|c: &CubieCube| HTRDRUDCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

fn gen_fr_leave_slice() -> PruningTable<{ fr::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord> {
    info!("Generating FRLS pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &CubieCube| FRUDNoSliceCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

fn gen_fr() -> PruningTable<{ fr::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord> {
    info!("Generating FR pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &CubieCube| FRUDWithSliceCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

fn gen_fr_finish() -> PruningTable<{ finish::FR_FINISH_SIZE }, FRUDFinishCoord> {
    info!("Generating FR finish pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&FRUD_FINISH_MOVESET, &|c: &CubieCube| FRUDFinishCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}