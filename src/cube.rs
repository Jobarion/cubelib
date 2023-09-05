use std::fmt::{Debug, Display, Formatter, Pointer, write};
use std::ops::{Index, IndexMut};
use std::str::FromStr;
use crate::algs::Algorithm;
use crate::cube::Face::*;
use crate::cube::Turn::*;

pub const FACES: [Face; 6] = [Up, Down, Front, Back, Left, Right];
pub const TURNS: [Turn; 3] = [Clockwise, CounterClockwise, Half];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Up = 0,
    Down = 1,
    Front = 2,
    Back = 3,
    Left = 4,
    Right = 5,
}

impl TryFrom<char> for Face {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'U' => Ok(Self::Up),
            'D' => Ok(Self::Down),
            'F' => Ok(Self::Front),
            'B' => Ok(Self::Back),
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            _ => Err(())
        }
    }
}

impl Into<char> for Face {
    fn into(self) -> char {
        match self {
            Up => 'U',
            Down => 'D',
            Front => 'F',
            Back => 'B',
            Left => 'L',
            Right => 'R',
        }
    }
}

impl<T, const N: usize> Index<Face> for [T; N] {
    type Output = T;

    fn index(&self, index: Face) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T, const N: usize> IndexMut<Face> for [T; N] {

    fn index_mut(&mut self, index: Face) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move(pub Face, pub Turn);

impl Move {
    pub fn invert(&self) -> Move {
        match *self {
            Move(face, Clockwise) => Move(face, CounterClockwise),
            Move(face, CounterClockwise) => Move(face, Clockwise),
            Move(face, Half) => Move(face, Half),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut face: String = String::from(<Face as Into<char>>::into(self.0));
        let turn = match self.1 {
            Clockwise => "",
            CounterClockwise => "'",
            Half => "2"
        };
        face.push_str(turn);
        write!(f, "{}", face)
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut chars = value.chars();
        let face = chars.next().map_or(Err(()), |c|Face::try_from(c))?;
        let turn = match chars.next() {
            Some('2') => Ok(Turn::Half),
            Some('\'') => Ok(Turn::CounterClockwise),
            None => Ok(Turn::Clockwise),
            _ => Err(())
        }?;
        if chars.next().is_none() {
            Ok(Move(face, turn))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White = 0,
    Yellow = 1,
    Green = 2,
    Blue = 3,
    Orange = 4,
    Red = 5,

    None =  6,
}

#[derive(Debug, Clone, Copy)]
pub enum CornerPosition {
    UBL = 0,
    UBR = 1,
    UFR = 2,
    UFL = 3,
    DFL = 4,
    DFR = 5,
    DBR = 6,
    DBL = 7
}

#[derive(Debug, Clone, Copy)]
pub enum EdgePosition {
    UB = 0,
    UR = 1,
    UF = 2,
    UL = 3,
    FR = 4,
    FL = 5,
    BR = 6,
    BL = 7,
    DF = 8,
    DR = 9,
    DB = 10,
    DL = 11
}

#[derive(Debug, Clone, Copy)]
pub struct Corner {
    pub id: u8,
    pub orientation: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub id: u8,
    pub oriented_ud: bool,
    pub oriented_fb: bool,
    pub oriented_rl: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Turn {
    Clockwise = 0,
    Half = 1,
    CounterClockwise = 2
}

pub trait Turnable {
    fn turn(&mut self, m: Move);
}


pub trait Invertible {
    fn invert(&mut self);
}

pub trait Cube: Display + Turnable + Invertible {
    fn new_solved() -> Self;
    fn get_facelets(&self) -> [[Color; 9]; 6];
    fn apply(&mut self, alg: &Algorithm) {
        for m in &alg.normal_moves {
            self.turn(*m);
        }
        self.invert();
        for m in &alg.inverse_moves {
            self.turn(*m);
        }
        self.invert();
    }
    fn fmt_display(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let facelets = self.get_facelets();
        let block_width = "   ";
        let block_spacing = " ";
        for x in (0..3).rev() {
            write!(f, "{}{}", block_width.repeat(3), block_spacing.repeat(4))?;
            for y in (0..3).rev() {
                write!(f, "{}{}", facelets[Face::Back][x * 3 + y], block_spacing)?;
            }
            writeln!(f)?;
        }
        for x in 0..3 {
            let x_rev = 2 - x;
            for y in (0..3).rev() {
                write!(f, "{}{}", facelets[Face::Left][x + y * 3], block_spacing)?;
            }
            write!(f, "{}", block_spacing)?;
            for y in 0..3 {
                write!(f, "{}{}", facelets[Face::Up][x * 3 + y], block_spacing)?;
            }
            write!(f, "{}", block_spacing)?;
            for y in 0..3 {
                write!(f, "{}{}", facelets[Face::Right][x_rev + y * 3], block_spacing)?;
            }
            write!(f, "{}", block_spacing)?;
            for y in (0..3).rev() {
                write!(f, "{}{}", facelets[Face::Down][x_rev * 3 + y], block_spacing)?;
            }
            writeln!(f)?;
        }

        for x in 0..3 {
            write!(f, "{}{}", block_width.repeat(3), block_spacing.repeat(4))?;
            for y in 0..3 {
                write!(f, "{}{}", facelets[Face::Front][x * 3 + y], block_spacing)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "⬜"),
            Color::Yellow => write!(f, "🟨"),
            Color::Blue => write!(f, "🟦"),
            Color::Green => write!(f, "🟩"),
            Color::Red => write!(f, "🟥"),
            Color::Orange => write!(f, "🟧"),
            Color::None => write!(f, "⬛"),
        }
    }
}