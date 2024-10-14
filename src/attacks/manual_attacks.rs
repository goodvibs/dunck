use std::cmp;
use crate::utils::Bitboard;
use crate::utils::Color;
use crate::utils::masks::*;

pub fn multi_knight_attacks(knights_mask: Bitboard) -> Bitboard {
    (knights_mask << 17 & !FILE_H) | (knights_mask << 15 & !FILE_A) | (knights_mask << 10 & !FILES_GH) | (knights_mask << 6 & !FILES_AB) |
        (knights_mask >> 17 & !FILE_A) | (knights_mask >> 15 & !FILE_H) | (knights_mask >> 10 & !FILES_AB) | (knights_mask >> 6 & !FILES_GH)
}

pub fn multi_king_attacks(kings_mask: Bitboard) -> Bitboard {
    (kings_mask << 9 & !FILE_H) | (kings_mask << 8) | (kings_mask << 7 & !FILE_A) |
        (kings_mask >> 9 & !FILE_A) | (kings_mask >> 8) | (kings_mask >> 7 & !FILE_H) |
        (kings_mask << 1 & !FILE_H) | (kings_mask >> 1 & !FILE_A)
}

pub fn multi_pawn_attacks(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => (pawns_mask << 9 & !FILE_H) | (pawns_mask << 7 & !FILE_A),
        Color::Black => (pawns_mask >> 7 & !FILE_H) | (pawns_mask >> 9 & !FILE_A)
    }
}

pub fn multi_pawn_moves(pawns_mask: Bitboard, by_color: Color) -> Bitboard {
    match by_color {
        Color::White => pawns_mask << 8,
        Color::Black => pawns_mask >> 8
    }
}

pub fn manual_single_rook_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros: u32 = src_mask.leading_zeros();
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
    let (mut pos_n, mut pos_s, mut pos_w, mut pos_e): (Bitboard, Bitboard, Bitboard, Bitboard) = (src_mask, src_mask, src_mask, src_mask);
    for _ in 0..n_distance {
        pos_n <<= 8;
        attacks |= pos_n;
        if occupied_mask & pos_n != 0 {
            break;
        }
    }
    for _ in 0..s_distance {
        pos_s >>= 8;
        attacks |= pos_s;
        if occupied_mask & pos_s != 0 {
            break;
        }
    }
    for _ in 0..w_distance {
        pos_w <<= 1;
        attacks |= pos_w;
        if occupied_mask & pos_w != 0 {
            break;
        }
    }
    for _ in 0..e_distance {
        pos_e >>= 1;
        attacks |= pos_e;
        if occupied_mask & pos_e != 0 {
            break;
        }
    }
    attacks
}

pub fn manual_single_bishop_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros: u32 = src_mask.leading_zeros();
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
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