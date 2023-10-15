use std::fmt::Display;

pub const NO_FLAG: u8 = 0b0000;
pub const EN_PASSANT_CAPTURE_FLAG: u8 = 0b0001;
pub const CASTLE_FLAG: u8 = 0b0010;
pub const PAWN_DOUBLE_MOVE_FLAG: u8 = 0b0011;
pub const PROMOTE_TO_QUEEN_FLAG: u8 = 0b0100;
pub const PROMOTE_TO_KNIGHT_FLAG: u8 = 0b0101;
pub const PROMOTE_TO_ROOK_FLAG: u8 = 0b0110;
pub const PROMOTE_TO_BISHOP_FLAG: u8 = 0b0111;

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
}