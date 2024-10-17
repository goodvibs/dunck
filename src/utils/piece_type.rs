use subenum::subenum;
use crate::utils::{Color, ColoredPiece};

#[subenum(SlidingPieceType)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    NoPieceType=0,
    Pawn=1,
    Knight=2,
    #[subenum(SlidingPieceType)]
    Bishop=3,
    #[subenum(SlidingPieceType)]
    Rook=4,
    Queen=5,
    King=6
}

impl PieceType {
    pub const LIMIT: u8 = 7;
    pub const AllPieceTypes: PieceType = PieceType::NoPieceType;

    pub const unsafe fn from(piece_type_number: u8) -> PieceType {
        assert!(piece_type_number < PieceType::LIMIT as u8, "Piece type number out of bounds");
        std::mem::transmute::<u8, PieceType>(piece_type_number)
    }

    pub const fn to_char(&self) -> char {
        ColoredPiece::from(Color::White, *self).to_char()
    }

    pub fn iter_all() -> impl Iterator<Item = PieceType> {
        PieceType::iter_between(PieceType::NoPieceType, PieceType::King)
    }

    pub fn iter_pieces() -> impl Iterator<Item = PieceType> {
        PieceType::iter_between(PieceType::Pawn, PieceType::King)
    }

    pub fn iter_between(first: PieceType, last: PieceType) -> impl Iterator<Item = PieceType> {
        (first as u8..=last as u8).map(|n| unsafe { PieceType::from(n) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_type() {
        assert_eq!(PieceType::NoPieceType as u8, 0);
        assert_eq!(PieceType::Pawn as u8, 1);
        assert_eq!(PieceType::AllPieceTypes as u8, 0);
        assert_eq!(PieceType::LIMIT, 7);
        unsafe {
            assert_eq!(PieceType::from(0), PieceType::NoPieceType);
            assert_eq!(PieceType::from(1), PieceType::Pawn);
            assert_eq!(PieceType::from(6), PieceType::King);
        }
    }
}