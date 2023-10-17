use std::fmt::Display;
use crate::squares::SQUARE_NAMES;

pub const NO_FLAG: u8 = 0;
pub const KNIGHT_MOVE_FLAG: u8 = 1;
pub const BISHOP_MOVE_FLAG: u8 = 2;
pub const ROOK_MOVE_FLAG: u8 = 3;
pub const QUEEN_MOVE_FLAG: u8 = 4;
pub const KING_MOVE_FLAG: u8 = 5;
pub const CASTLE_FLAG: u8 = 6;
pub const PAWN_MOVE_FLAG: u8 = 8;
pub const EN_PASSANT_FLAG: u8 = 10;
pub const PAWN_DOUBLE_MOVE_FLAG: u8 = 11;
pub const PROMOTE_TO_QUEEN_FLAG: u8 = 12;
pub const PROMOTE_TO_KNIGHT_FLAG: u8 = 13;
pub const PROMOTE_TO_ROOK_FLAG: u8 = 14;
pub const PROMOTE_TO_BISHOP_FLAG: u8 = 15;

#[derive(Copy, Clone)]
pub struct Move {
    // format: FFFFTTTTTTSSSSSS
    // F = flag, T = target square, S = source square
    pub value: u16,
}

impl Move {
    pub fn new(src: u32, dst: u32, flag: u8) -> Move {
        Move {
            value: ((flag as u16) << 12) | ((dst as u16) << 6) | (src as u16)
        }
    }

    pub fn unpack(&self) -> (u32, u32, u8) {
        let src: u32 = (self.value & 0b0000000000111111) as u32;
        let dst: u32 = ((self.value & 0b0000111111000000) >> 6) as u32;
        let flag: u8 = ((self.value & 0b1111000000000000) >> 12) as u8;
        (src, dst, flag)
    }

    pub fn to_readable(&self) -> (&str, &str, &str) {
        let (src, dst, flag) = self.unpack();
        let src_str = SQUARE_NAMES[src as usize];
        let dst_str = SQUARE_NAMES[dst as usize];
        let flag_str = match flag {
            PAWN_MOVE_FLAG => "P",
            PAWN_DOUBLE_MOVE_FLAG => "P2",
            EN_PASSANT_FLAG => "Px",
            KNIGHT_MOVE_FLAG => "N",
            BISHOP_MOVE_FLAG => "B",
            ROOK_MOVE_FLAG => "R",
            QUEEN_MOVE_FLAG => "Q",
            KING_MOVE_FLAG => "K",
            CASTLE_FLAG => "castling",
            PROMOTE_TO_QUEEN_FLAG => "P to Q",
            PROMOTE_TO_KNIGHT_FLAG => "P to N",
            PROMOTE_TO_ROOK_FLAG => "P to R",
            PROMOTE_TO_BISHOP_FLAG => "P to B",
            _ => ""
        };
        (src_str, dst_str, flag_str)
    }
}