use std::fmt;
use crate::consts::SQUARE_NAMES;

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

    pub fn matches(&self, move_str: &str) -> bool {
        if move_str.len() < 2 {
            return false;
        }
        let (src_str, dst_str, flag_str) = self.to_readable();
        if move_str == "0-0" || move_str == "O-O" {
            return flag_str == "castling" && dst_str.starts_with('g');
        }
        if move_str == "0-0-0" || move_str == "O-O-O" {
            return flag_str == "castling" && dst_str.starts_with('c');
        }
        let bytes = move_str.as_bytes();
        let mut end = move_str.len() - move_str.ends_with('+') as usize - move_str.ends_with('#') as usize;
        if bytes[end - 1].is_ascii_uppercase() {
            if flag_str != "P to ?".replace('?', &move_str[end - 1..end]) {
                return false;
            }
            end -= (bytes[end - 2] == b'=') as usize;
        }
        let is_capture = move_str.contains('x');
        if &move_str[end - 2..end] != dst_str {
            return false;
        }
        let is_piece_move = bytes[0].is_ascii_uppercase();
        if is_piece_move {
            if flag_str != &move_str[0..1] {
                return false;
            }
        }
        else {
            if !flag_str.contains('P') {
                return false;
            }
        }
        return match end - is_piece_move as usize - is_capture as usize {
            2 => true,
            3 => src_str.contains(bytes[is_piece_move as usize] as char),
            _ => false
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (src_str, dst_str, flag_str) = self.to_readable();
        write!(f, "{}{}{}", src_str, dst_str, flag_str)
    }
}