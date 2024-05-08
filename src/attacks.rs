use crate::bitboard::Bitboard;
use crate::enums::Color;
use crate::manual_attacks;

pub fn knight_attacks(knights: Bitboard) -> Bitboard {
    manual_attacks::knight_attacks(knights)
}

pub fn king_attacks(kings: Bitboard) -> Bitboard {
    manual_attacks::king_attacks(kings)
}

pub fn pawn_attacks(pawns: Bitboard, color: Color) -> Bitboard {
    manual_attacks::pawn_attacks(pawns, color)
}

pub fn pawn_moves(pawns: Bitboard, color: Color) -> Bitboard {
    manual_attacks::pawn_moves(pawns, color)
}

pub fn rook_attacks(origin: Bitboard, occupied: Bitboard) -> Bitboard {
    manual_attacks::rook_attacks(origin, occupied)
}

pub fn bishop_attacks(origin: Bitboard, occupied: Bitboard) -> Bitboard {
    manual_attacks::bishop_attacks(origin, occupied)
}