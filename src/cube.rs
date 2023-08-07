use std::ops::Index;


#[derive(Debug, Clone, Copy)]
pub enum Face {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl From<u32> for Face {
    fn from(face: u32) -> Self {
        match face {
            0 => Face::Up,
            1 => Face::Down,
            2 => Face::Front,
            3 => Face::Back,
            4 => Face::Left,
            5 => Face::Right,
            _ => panic!("Invalid face")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Clockwise = 0,
    Half = 1,
    CounterClockwise = 2
}

pub trait Cube {
    fn turn(&mut self, face: Face, turn_type: Turn);
}