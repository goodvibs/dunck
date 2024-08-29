use crate::bitboard::Bitboard;
use crate::miscellaneous::Color;
use crate::{manual_attacks, precompute};

pub fn single_knight_attacks(src_mask: Bitboard) -> Bitboard {
    precompute::single_knight_attacks(src_mask)
}

pub fn single_king_attacks(src_mask: Bitboard) -> Bitboard {
    precompute::single_king_attacks(src_mask)
}

pub fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    manual_attacks::multi_knight_attacks(knights_mask)
}

pub fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    manual_attacks::multi_king_attacks(kings_mask)
}

pub fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual_attacks::multi_pawn_attacks(pawns_mask, by_color)
}

pub fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual_attacks::multi_pawn_moves(pawns_mask, by_color)
}

pub fn single_rook_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    manual_attacks::single_rook_attacks(src_mask, occupied_mask)
}

pub fn single_bishop_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    manual_attacks::single_bishop_attacks(src_mask, occupied_mask)
}