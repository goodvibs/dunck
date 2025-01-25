use std::fmt::Display;
use crate::utils::{Bitboard, Color};
use crate::utils::charboard::SQUARE_NAMES;
use crate::utils::masks::{FILES, RANKS};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Square {
    A8=0, B8=1, C8=2, D8=3, E8=4, F8=5, G8=6, H8=7,
    A7=8, B7=9, C7=10, D7=11, E7=12, F7=13, G7=14, H7=15,
    A6=16, B6=17, C6=18, D6=19, E6=20, F6=21, G6=22, H6=23,
    A5=24, B5=25, C5=26, D5=27, E5=28, F5=29, G5=30, H5=31,
    A4=32, B4=33, C4=34, D4=35, E4=36, F4=37, G4=38, H4=39,
    A3=40, B3=41, C3=42, D3=43, E3=44, F3=45, G3=46, H3=47,
    A2=48, B2=49, C2=50, D2=51, E2=52, F2=53, G2=54, H2=55,
    A1=56, B1=57, C1=58, D1=59, E1=60, F1=61, G1=62, H1=63
}

const ALL: [Square; 64] = [
    Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
    Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
    Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
    Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
    Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
    Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
    Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
    Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1
];

impl Square {
    pub const unsafe fn from(square_number: u8) -> Square {
        assert!(square_number < 64, "Square number out of bounds");
        std::mem::transmute::<u8, Square>(square_number)
    }
    
    pub const unsafe fn from_rank_file(rank: u8, file: u8) -> Square {
        assert!(rank < 8 && file < 8, "Rank or file out of bounds");
        std::mem::transmute::<u8, Square>((7 - rank) * 8 + file)
    }

    pub const fn get_mask(&self) -> Bitboard {
        1 << (63 - *self as u8)
    }

    pub const fn get_file(&self) -> u8 {
        *self as u8 % 8
    }

    pub const fn get_file_mask(&self) -> Bitboard {
        FILES[self.get_file() as usize]
    }

    pub const fn get_rank(&self) -> u8 {
        7 - *self as u8 / 8
    }

    pub const fn get_rank_mask(&self) -> Bitboard {
        RANKS[self.get_rank() as usize]
    }
    
    pub const fn up(&self) -> Option<Square> {
        if self.get_rank() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 8) })
        }
    }
    
    pub const fn down(&self) -> Option<Square> {
        if self.get_rank() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 8) })
        }
    }
    
    pub const fn left(&self) -> Option<Square> {
        if self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 1) })
        }
    }
    
    pub const fn right(&self) -> Option<Square> {
        if self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 1) })
        }
    }
    
    pub const fn up_left(&self) -> Option<Square> {
        if self.get_rank() == 7 || self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 9) })
        }
    }
    
    pub const fn up_right(&self) -> Option<Square> {
        if self.get_rank() == 7 || self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 7) })
        }
    }
    
    pub const fn down_left(&self) -> Option<Square> {
        if self.get_rank() == 0 || self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 7) })
        }
    }
    
    pub const fn down_right(&self) -> Option<Square> {
        if self.get_rank() == 0 || self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 9) })
        }
    }
    
    pub const fn reflect_rank(&self) -> Square {
        unsafe { Square::from((self.get_rank() * 8) + self.get_file()) }
    }
    
    pub const fn rotated_perspective(&self) -> Square {
        unsafe { Square::from(63 - *self as u8) }
    }
    
    pub const fn to_perspective_from_white(&self, desired_perspective: Color) -> Square {
        match desired_perspective {
            Color::White => *self,
            Color::Black => self.rotated_perspective()
        }
    }
    
    pub const fn to_perspective_from_black(&self, desired_perspective: Color) -> Square {
        match desired_perspective {
            Color::White => self.rotated_perspective(),
            Color::Black => *self
        }
    }

    pub const fn get_file_char(&self) -> char {
        (b'a' + self.get_file()) as char
    }

    pub const fn get_rank_char(&self) -> char {
        (b'1' + self.get_rank()) as char
    }

    pub const fn readable(&self) -> &str {
        SQUARE_NAMES[*self as usize]
    }

    pub fn iter_all() -> impl Iterator<Item = &'static Square> {
        ALL.iter()
    }

    pub fn iter_between(first: Square, last: Square) -> impl Iterator<Item = &'static Square> {
        ALL[first as usize..=last as usize].iter()
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.readable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square() {
        assert_eq!(Square::A8 as u8, 0);
        assert_eq!(Square::H8 as u8, 7);
        assert_eq!(Square::A1 as u8, 56);
        assert_eq!(Square::H1 as u8, 63);
    }
}