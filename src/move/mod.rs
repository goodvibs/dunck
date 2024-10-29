mod move_flag;
mod san;

pub use move_flag::*;
pub use san::*;

use crate::utils::{PieceType, Square};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    // format: DDDDDDSSSSSSPPMM (D: destination, S: source, P: promotion PieceType value minus 2, M: MoveFlag value)
    pub value: u16,
}

impl Move {
    pub const DEFAULT_PROMOTION_VALUE: PieceType = PieceType::Rook;

    pub fn new(dst: Square, src: Square, promotion: PieceType, flag: MoveFlag) -> Move {
        assert!(promotion != PieceType::King && promotion != PieceType::Pawn, "Invalid promotion piece type");
        Move {
            value: ((dst as u16) << 10) | ((src as u16) << 4) | ((promotion as u16 - 2) << 2) | flag as u16
        }
    }

    pub fn new_non_promotion(dst: Square, src: Square, flag: MoveFlag) -> Move {
        Move::new(dst, src, Move::DEFAULT_PROMOTION_VALUE, flag)
    }

    pub const fn get_destination(&self) -> Square {
        let dst_int = (self.value >> 10) as u8;
        unsafe { Square::from(dst_int) }
    }

    pub const fn get_source(&self) -> Square {
        let src_int = ((self.value & 0b0000001111110000) >> 4) as u8;
        unsafe { Square::from(src_int) }
    }

    pub const fn get_promotion(&self) -> PieceType {
        let promotion_int = ((self.value & 0b0000000000001100) >> 2) as u8;
        unsafe { PieceType::from(promotion_int + 2) }
    }

    pub const fn get_flag(&self) -> MoveFlag {
        let flag_int = (self.value & 0b0000000000000011) as u8;
        unsafe { MoveFlag::from(flag_int) }
    }

    pub const fn unpack(&self) -> (Square, Square, PieceType, MoveFlag) {
        (self.get_destination(), self.get_source(), self.get_promotion(), self.get_flag())
    }

    pub fn readable(&self) -> String {
        let (dst, src, promotion, flag) = self.unpack();
        let (dst_str, src_str, promotion_char, flag_str) = (src.readable(), dst.readable(), promotion.to_char(), flag.to_readable());
        format!("{}{}{}", dst_str, src_str, flag_str.replace('?', &promotion_char.to_string()))
    }

    pub fn uci(&self) -> String {
        let (dst, src, promotion, flag) = self.unpack();
        let (dst_str, src_str) = (dst.readable(), src.readable());
        let promotion_str = match flag {
            MoveFlag::Promotion => promotion.to_char().to_string(),
            _ => "".to_string()
        };
        format!("{}{}{}", src_str, dst_str, promotion_str)
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.readable())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveFlag};
    use crate::utils::{PieceType, Square};

    #[test]
    fn test_move() {
        for dst_square in Square::iter_all() {
            for src_square in Square::iter_all() {
                for promotion_piece in PieceType::iter_middle_pieces() {
                    for flag_int in 0..4 {
                        let flag = unsafe { MoveFlag::from(flag_int) };

                        let mv = Move::new(dst_square, src_square, promotion_piece, flag);
                        assert_eq!(mv.get_destination(), dst_square);
                        assert_eq!(mv.get_source(), src_square);
                        assert_eq!(mv.get_promotion(), promotion_piece);
                        assert_eq!(mv.get_flag(), flag);
                    }
                }
            }
        }
    }

    #[test]
    fn test_san() {
        // todo: implement
    }
}