use crate::cube::{Face, Move, Turn};
use crate::df_search::LEGAL_MOVE_COUNT;

pub struct MoveSet {
    pub st_moves: &'static [Move],
    pub aux_moves: &'static [Move],
    pub transitions: [TransitionTable; LEGAL_MOVE_COUNT],
}

#[derive(Copy, Clone)]
pub struct TransitionTable {
    pub allowed: u32,
    pub can_end: u32,
}

#[derive(Debug)]
pub struct Transition {
    pub allowed: bool,
    pub can_end: bool,
}

impl Transition {
    pub fn any() -> Self {
        Transition {
            allowed: true,
            can_end: true,
        }
    }
}

impl TransitionTable {
    pub const ANY: u32 = 0x3FFFF;
    pub const NONE: u32 = 0;

    //This enforces an order between [UFL] and [DBR] moves to avoid duplicates. U D is allowed, D U is not.
    pub const DEFAULT_ALLOWED_AFTER: [u32; 6] = [
        TransitionTable::except_faces_to_mask([Face::Up]), //U
        TransitionTable::except_faces_to_mask([Face::Up, Face::Down]), //D
        TransitionTable::except_faces_to_mask([Face::Front]), //F
        TransitionTable::except_faces_to_mask([Face::Front, Face::Back]), //B
        TransitionTable::except_faces_to_mask([Face::Left]), //L
        TransitionTable::except_faces_to_mask([Face::Left, Face::Right]), //R
    ];

    pub const fn moves_to_mask<const L: usize>(moves: [Move; L]) -> u32 {
        let mut mask = 0u32;
        let mut i = 0;
        while i < L {
            mask |= 1 << moves[i].to_id();
            i += 1;
        }
        mask & 0x3FFFF
    }

    pub const fn except_moves_to_mask<const L: usize>(moves: [Move; L]) -> u32 {
        !TransitionTable::moves_to_mask(moves) & 0x3FFFF
    }

    pub const fn faces_to_mask<const L: usize>(faces: [Face; L]) -> u32 {
        let mut mask = 0u32;
        let mut i = 0;
        while i < L {
            let f = faces[i];
            mask |= 1 << Move(f, Turn::Clockwise).to_id();
            mask |= 1 << Move(f, Turn::CounterClockwise).to_id();
            mask |= 1 << Move(f, Turn::Half).to_id();
            i += 1;
        }
        mask & 0x3FFFF
    }

    pub const fn except_faces_to_mask<const L: usize>(faces: [Face; L]) -> u32 {
        !TransitionTable::faces_to_mask(faces) & 0x3FFFF
    }

    pub const fn new(allowed: u32, can_end: u32) -> Self {
        TransitionTable { allowed, can_end }
    }

    pub fn check_move(&self, m: &Move) -> Transition {
        let mid = m.to_id() as u32;
        let allowed = self.allowed & (1 << mid) != 0;
        let can_end = self.can_end & (1 << mid) != 0;
        Transition { allowed, can_end }
    }
}
