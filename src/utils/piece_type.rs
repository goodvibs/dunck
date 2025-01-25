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

const ALL: [PieceType; 7] = [
    PieceType::NoPieceType,
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King
];

const ALL_PIECES: [PieceType; 6] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King
];

const ALL_NON_KING_PIECES: [PieceType; 5] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen
];

const ALL_SLIDING_PIECES: [PieceType; 3] = [
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen
];

const ALL_NON_SLIDING_PIECES: [PieceType; 3] = [
    PieceType::Pawn,
    PieceType::Knight,
    PieceType::King
];

const ALL_NON_PAWN_PIECES: [PieceType; 5] = [
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen,
    PieceType::King
];

const ALL_PROMOTION_PIECES: [PieceType; 4] = [
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Rook,
    PieceType::Queen
];

impl PieceType {
    pub const LIMIT: u8 = 7;
    pub const AllPieceTypes: PieceType = PieceType::NoPieceType;

    pub const unsafe fn from(piece_type_number: u8) -> PieceType {
        assert!(piece_type_number < PieceType::LIMIT, "Piece type number out of bounds");
        std::mem::transmute::<u8, PieceType>(piece_type_number)
    }

    pub const fn to_char(&self) -> char {
        ColoredPiece::from(Color::White, *self).to_char()
    }

    pub fn iter_all() -> impl Iterator<Item = PieceType> {
        ALL.iter().copied()
    }

    pub fn iter_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_PIECES.iter()
    }
    
    pub fn iter_non_king_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_NON_KING_PIECES.iter()
    }
    
    pub fn iter_sliding_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_SLIDING_PIECES.iter()
    }
    
    pub fn iter_non_sliding_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_NON_SLIDING_PIECES.iter()
    }
    
    pub fn iter_non_pawn_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_NON_PAWN_PIECES.iter()
    }
    
    pub fn iter_promotion_pieces() -> impl Iterator<Item = &'static PieceType> {
        ALL_PROMOTION_PIECES.iter()
    }
    
    pub fn iter_between(first: PieceType, last: PieceType) -> impl Iterator<Item = &'static PieceType> {
        ALL[first as usize..=last as usize].iter()
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