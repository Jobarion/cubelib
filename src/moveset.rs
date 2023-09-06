use crate::cube::{Face, FACES, Move, Turn, TURNS};
use crate::df_search::{ALL_MOVES, LEGAL_MOVE_COUNT};

pub struct MoveSet<const ST_Size: usize, const Aux_Size: usize> {
    pub st_moves: [Move; ST_Size],
    pub aux_moves: [Move; Aux_Size],
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
        Transition { allowed: true, can_end: true }
    }
}

impl TransitionTable {

    pub const ANY: u32 = 0x3FFFF;

    pub const DEFAULT_ALLOWED_AFTER: [u32; 6] = [
        TransitionTable::except_faces_to_mask([Face::Up]), //U
        TransitionTable::except_faces_to_mask([Face::Up, Face::Down]), //D
        TransitionTable::except_faces_to_mask([Face::Front]), //F
        TransitionTable::except_faces_to_mask([Face::Front, Face::Back]), //B
        TransitionTable::except_faces_to_mask([Face::Left]), //L
        TransitionTable::except_faces_to_mask([Face::Left, Face::Right]), //R
    ];

    pub const fn except_moves_to_mask<const L: usize>(moves: [Move; L]) -> u32 {
        let mut mask = 0u32;
        let mut i = 0;
        while i < L {
            mask |= 1 << moves[i].to_id();
            i += 1;
        }
        !mask & 0x3FFFF
    }

    pub const fn except_faces_to_mask<const L: usize>(faces: [Face; L]) -> u32 {
        let mut mask = 0u32;
        let mut i = 0;
        while i < L {
            let f = faces[i];
            mask |= 1 << Move(f, Turn::Clockwise).to_id();
            mask |= 1 << Move(f, Turn::CounterClockwise).to_id();
            mask |= 1 << Move(f, Turn::Half).to_id();
            i += 1;
        }
        !mask & 0x3FFFF
    }

    pub const fn new(allowed: u32, can_end: u32) -> Self {
        TransitionTable { allowed, can_end }
    }

    pub fn check_move(&self, m: &Move) -> Transition {
        let mid = m.to_id() as u32;
        let allowed = self.allowed & (1 << mid) != 0;
        let can_end = self.can_end & (1 << mid) != 0;
        Transition{allowed, can_end}
    }


}