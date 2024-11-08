///! This module contains functions to calculate attack bitboards for different pieces.

mod magic;
mod manual;
mod precomputed;

use crate::utils::{Bitboard, Square};
use crate::utils::Color;

pub fn single_knight_attacks(src_square: Square) -> Bitboard {
    precomputed::precomputed_single_knight_attacks(src_square)
}

pub fn single_king_attacks(src_square: Square) -> Bitboard {
    precomputed::precomputed_single_king_attacks(src_square)
}

pub fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    manual::multi_knight_attacks(knights_mask)
}

pub fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    manual::multi_king_attacks(kings_mask)
}

pub fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual::multi_pawn_attacks(pawns_mask, by_color)
}

pub fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    manual::multi_pawn_moves(pawns_mask, by_color)
}

pub fn single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    magic::magic_single_rook_attacks(src_square, occupied_mask)
}

pub fn single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    magic::magic_single_bishop_attacks(src_square, occupied_mask)
}