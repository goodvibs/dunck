//! Contains functions that manually calculate attacks for all pieces

use std::cmp;
use crate::utils::{Bitboard, Square};
use crate::utils::Color;
use crate::utils::masks::*;

/// Returns a bitboard with all squares attacked by knights indicated by the bits in `knights_mask`
pub fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    (knights_mask << 17 & !FILE_H) | (knights_mask << 15 & !FILE_A) | (knights_mask << 10 & !FILES_GH) | (knights_mask << 6 & !FILES_AB) |
        (knights_mask >> 17 & !FILE_A) | (knights_mask >> 15 & !FILE_H) | (knights_mask >> 10 & !FILES_AB) | (knights_mask >> 6 & !FILES_GH)
}

/// Returns a bitboard with all squares attacked by kings indicated by the bits in `kings_mask`
pub fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    (kings_mask << 9 & !FILE_H) | (kings_mask << 8) | (kings_mask << 7 & !FILE_A) |
        (kings_mask >> 9 & !FILE_A) | (kings_mask >> 8) | (kings_mask >> 7 & !FILE_H) |
        (kings_mask << 1 & !FILE_H) | (kings_mask >> 1 & !FILE_A)
}

/// Returns a bitboard with all squares attacked by pawns indicated by the bits in `pawns_mask`
pub fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => (pawns_mask << 9 & !FILE_H) | (pawns_mask << 7 & !FILE_A),
        Color::Black => (pawns_mask >> 7 & !FILE_H) | (pawns_mask >> 9 & !FILE_A)
    }
}

/// Returns a bitboard with all squares that pawns indicated by the bits in `pawns_mask` can move to
pub fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => pawns_mask << 8,
        Color::Black => pawns_mask >> 8
    }
}

/// Returns a bitboard with all squares attacked by a rook on `src_square` 
/// with `occupied_mask` as the mask of occupied squares
pub fn manual_single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    let src_square_mask = src_square.get_mask();
    let mut result: Bitboard = 0;

    let mut mask = src_square_mask << 1;
    while mask != 0 && mask & FILE_H == 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask <<= 1;
    }

    let mut mask = src_square_mask << 8;
    while mask != 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask <<= 8;
    }

    let mut mask = src_square_mask >> 1;
    while mask != 0 && mask & FILE_A == 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask >>= 1;
    }

    let mut mask = src_square_mask >> 8;
    while mask != 0 {
        result |= mask;
        if occupied_mask & mask != 0 {
            break;
        }
        mask >>= 8;
    }
    
    result
}

/// Returns a bitboard with all squares attacked by a bishop on `src_square` 
/// with `occupied_mask` as the mask of occupied squares
pub fn manual_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros = src_square as u32;
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
    let src_mask = src_square.get_mask();
    let (mut pos_nw, mut pos_ne, mut pos_sw, mut pos_se): (Bitboard, Bitboard, Bitboard, Bitboard) = (src_mask, src_mask, src_mask, src_mask);
    for _ in 0..cmp::min(n_distance, w_distance) {
        pos_nw <<= 9;
        attacks |= pos_nw;
        if occupied_mask & pos_nw != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(n_distance, e_distance) {
        pos_ne <<= 7;
        attacks |= pos_ne;
        if occupied_mask & pos_ne != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(s_distance, w_distance) {
        pos_sw >>= 7;
        attacks |= pos_sw;
        if occupied_mask & pos_sw != 0 {
            break;
        }
    }
    for _ in 0..cmp::min(s_distance, e_distance) {
        pos_se >>= 9;
        attacks |= pos_se;
        if occupied_mask & pos_se != 0 {
            break;
        }
    }
    attacks
}