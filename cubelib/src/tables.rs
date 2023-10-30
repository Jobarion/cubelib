use std::time::Instant;
use log::{debug, info};
#[cfg(feature = "step_dr")]
use crate::steps::dr::coords::DRUDEOFBCoord;
#[cfg(feature = "step_eo")]
use crate::steps::eo::coords::EOCoordFB;
#[cfg(feature = "step_finish")]
use crate::steps::finish::coords::FRUDFinishCoord;
#[cfg(feature = "step_fr")]
use crate::steps::fr::coords::{FRUDNoSliceCoord, FRUDWithSliceCoord};
#[cfg(feature = "step_htr")]
use crate::steps::htr::coords::HTRDRUDCoord;
use crate::cubie::{CubieCube, EdgeCubieCube};
use crate::lookup_table;
use crate::lookup_table::PruningTable;
#[cfg(feature = "step_dr")]
use crate::steps::dr::dr_config::{DR_UD_EO_FB_MOVESET, DRPruningTable};
#[cfg(feature = "step_eo")]
use crate::steps::eo::eo_config::{EO_FB_MOVESET, EOPruningTable};
#[cfg(feature = "step_finish")]
use crate::steps::finish::finish_config::{FRFinishPruningTable, FRUD_FINISH_MOVESET};
#[cfg(feature = "step_fr")]
use crate::steps::fr::fr_config::{FR_UD_MOVESET, FRLeaveSlicePruningTable, FRPruningTable};
#[cfg(feature = "step_htr")]
use crate::steps::htr::htr_config::{HTR_DR_UD_MOVESET, HTRPruningTable};

pub struct PruningTables {
    #[cfg(feature = "step_eo")]
    eo: Option<EOPruningTable>,
    #[cfg(feature = "step_dr")]
    dr: Option<DRPruningTable>,
    #[cfg(feature = "step_htr")]
    htr: Option<HTRPruningTable>,
    #[cfg(feature = "step_fr")]
    frls: Option<FRLeaveSlicePruningTable>,
    #[cfg(feature = "step_fr")]
    fr: Option<FRPruningTable>,
    #[cfg(feature = "step_finish")]
    fr_finish: Option<FRFinishPruningTable>
}


impl PruningTables {

    pub fn new() -> PruningTables {
        PruningTables {
            #[cfg(feature = "step_eo")]
            eo: None,
            #[cfg(feature = "step_dr")]
            dr: None,
            #[cfg(feature = "step_htr")]
            htr: None,
            #[cfg(feature = "step_fr")]
            frls: None,
            #[cfg(feature = "step_fr")]
            fr: None,
            #[cfg(feature = "step_finish")]
            fr_finish: None
        }
    }

    #[cfg(feature = "step_eo")]
    pub fn gen_eo(&mut self) {
        if self.eo.is_none() {
            let table = gen_eo();
            self.eo = Some(table);
        }
    }

    #[cfg(feature = "step_eo")]
    pub fn eo(&self) -> Option<&EOPruningTable> {
        self.eo.as_ref()
    }

    #[cfg(feature = "step_dr")]
    pub fn gen_dr(&mut self) {
        if self.dr.is_none() {
            let table = gen_dr();
            self.dr = Some(table);
        }
    }

    #[cfg(feature = "step_dr")]
    pub fn dr(&self) -> Option<&DRPruningTable> {
        self.dr.as_ref()
    }

    #[cfg(feature = "step_htr")]
    pub fn gen_htr(&mut self) {
        if self.htr.is_none() {
            let table = gen_htr();
            self.htr = Some(table);
        }
    }

    #[cfg(feature = "step_htr")]
    pub fn htr(&self) -> Option<&HTRPruningTable> {
        self.htr.as_ref()
    }

    #[cfg(feature = "step_fr")]
    pub fn gen_fr_leave_slice(&mut self) {
        if self.frls.is_none() {
            let table = gen_fr_leave_slice();
            self.frls = Some(table);
        }
    }

    #[cfg(feature = "step_fr")]
    pub fn fr_leave_slice(&self) -> Option<&FRLeaveSlicePruningTable> {
        self.frls.as_ref()
    }

    #[cfg(feature = "step_fr")]
    pub fn gen_fr(&mut self) {
        if self.fr.is_none() {
            let table = gen_fr();
            self.fr = Some(table);
        }
    }

    #[cfg(feature = "step_fr")]
    pub fn fr(&self) -> Option<&FRPruningTable> {
        self.fr.as_ref()
    }

    #[cfg(feature = "step_finish")]
    pub fn gen_fr_finish(&mut self) {
        if self.fr_finish.is_none() {
            let table = gen_fr_finish();
            self.fr_finish = Some(table);
        }
    }

    #[cfg(feature = "step_finish")]
    pub fn fr_finish(&self) -> Option<&FRFinishPruningTable> {
        self.fr_finish.as_ref()
    }
}

#[cfg(feature = "step_eo")]
fn gen_eo() -> PruningTable<2048, EOCoordFB> {
    info!("Generating EO pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&EO_FB_MOVESET, &|c: &EdgeCubieCube| EOCoordFB::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "step_dr")]
fn gen_dr() -> PruningTable<{ crate::steps::dr::coords::DRUDEOFB_SIZE }, DRUDEOFBCoord> {
    info!("Generating DR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&DR_UD_EO_FB_MOVESET, &|c: &CubieCube| DRUDEOFBCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "step_htr")]
fn gen_htr() -> PruningTable<{ crate::steps::htr::coords::HTRDRUD_SIZE }, HTRDRUDCoord> {
    info!("Generating HTR pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&HTR_DR_UD_MOVESET, &|c: &CubieCube| HTRDRUDCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "step_fr")]
fn gen_fr_leave_slice() -> PruningTable<{ crate::steps::fr::coords::FRUD_NO_SLICE_SIZE }, FRUDNoSliceCoord> {
    info!("Generating FRLS pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &CubieCube| FRUDNoSliceCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "step_fr")]
fn gen_fr() -> PruningTable<{ crate::steps::fr::coords::FRUD_WITH_SLICE_SIZE }, FRUDWithSliceCoord> {
    info!("Generating FR pruning table...");
    let time = Instant::now();
    let table = lookup_table::generate(&FR_UD_MOVESET, &|c: &CubieCube| FRUDWithSliceCoord::from(c));
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}

#[cfg(feature = "step_finish")]
fn gen_fr_finish() -> PruningTable<{ crate::steps::finish::coords::FR_FINISH_SIZE }, FRUDFinishCoord> {
    info!("Generating FR finish pruning table...");
    #[cfg(not(target_arch = "wasm32"))]
    let time = Instant::now();
    let table = lookup_table::generate(&FRUD_FINISH_MOVESET, &|c: &CubieCube| FRUDFinishCoord::from(c));
    #[cfg(not(target_arch = "wasm32"))]
    debug!("Took {}ms", time.elapsed().as_millis());
    table
}