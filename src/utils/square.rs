use std::fmt::Display;
use crate::utils::Bitboard;
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

impl Square {
    pub const unsafe fn from(square_number: u8) -> Square {
        assert!(square_number < 64, "Square number out of bounds");
        std::mem::transmute::<u8, Square>(square_number)
    }

    pub const fn to_mask(&self) -> Bitboard {
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

    pub const fn get_file_char(&self) -> char {
        (b'a' + self.get_file()) as char
    }

    pub const fn get_rank_char(&self) -> char {
        (b'1' + self.get_rank()) as char
    }

    pub const fn readable(&self) -> &str {
        SQUARE_NAMES[*self as usize]
    }

    pub fn iter_all() -> impl Iterator<Item = Square> {
        Square::iter_between(Square::A8, Square::H1)
    }

    pub fn iter_between(first: Square, last: Square) -> impl Iterator<Item = Square> {
        (first as u8..=last as u8).map(|n| unsafe { Square::from(n) })
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