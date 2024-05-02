use std::cmp;
use crate::charboard::*;
use crate::masks::*;

pub fn knight_attacks(knights: Bitboard) -> Bitboard {
    (knights << 17 & !FILE_H) | (knights << 15 & !FILE_A) | (knights << 10 & !FILES_GH) | (knights << 6 & !FILES_AB) |
        (knights >> 17 & !FILE_A) | (knights >> 15 & !FILE_H) | (knights >> 10 & !FILES_AB) | (knights >> 6 & !FILES_GH)
}

pub fn king_attacks(kings: Bitboard) -> Bitboard {
    (kings << 9 & !FILE_H) | (kings << 8) | (kings << 7 & !FILE_A) |
        (kings >> 9 & !FILE_A) | (kings >> 8) | (kings >> 7 & !FILE_H) |
        (kings << 1 & !FILE_H) | (kings >> 1 & !FILE_A)
}

pub fn pawn_attacks(pawns: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => (pawns << 9 & !FILE_H) | (pawns << 7 & !FILE_A),
        Color::Black => (pawns >> 7 & !FILE_H) | (pawns >> 9 & !FILE_A)
    }
}

pub fn pawn_moves(pawns: Bitboard, color: Color) -> Bitboard {
    match color {
        Color::White => pawns << 8,
        Color::Black => pawns >> 8
    }
}

pub fn rook_attacks(origin: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros: u32 = origin.leading_zeros();
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
    let (mut pos_n, mut pos_s, mut pos_w, mut pos_e): (Bitboard, Bitboard, Bitboard, Bitboard) = (origin, origin, origin, origin);
    for i in 0..n_distance {
        pos_n <<= 8;
        attacks |= pos_n;
        if occupied & pos_n != 0 {
            break;
        }
    }
    for i in 0..s_distance {
        pos_s >>= 8;
        attacks |= pos_s;
        if occupied & pos_s != 0 {
            break;
        }
    }
    for i in 0..w_distance {
        pos_w <<= 1;
        attacks |= pos_w;
        if occupied & pos_w != 0 {
            break;
        }
    }
    for i in 0..e_distance {
        pos_e >>= 1;
        attacks |= pos_e;
        if occupied & pos_e != 0 {
            break;
        }
    }
    attacks
}

pub fn bishop_attacks(origin: Bitboard, occupied: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;
    let leading_zeros: u32 = origin.leading_zeros();
    let n_distance: u32 = leading_zeros / 8;
    let s_distance: u32 = 7 - n_distance;
    let w_distance: u32 = leading_zeros % 8;
    let e_distance: u32 = 7 - w_distance;
    let (mut pos_nw, mut pos_ne, mut pos_sw, mut pos_se): (Bitboard, Bitboard, Bitboard, Bitboard) = (origin, origin, origin, origin);
    for i in 0..cmp::min(n_distance, w_distance) {
        pos_nw <<= 9;
        attacks |= pos_nw;
        if occupied & pos_nw != 0 {
            break;
        }
    }
    for i in 0..cmp::min(n_distance, e_distance) {
        pos_ne <<= 7;
        attacks |= pos_ne;
        if occupied & pos_ne != 0 {
            break;
        }
    }
    for i in 0..cmp::min(s_distance, w_distance) {
        pos_sw >>= 7;
        attacks |= pos_sw;
        if occupied & pos_sw != 0 {
            break;
        }
    }
    for i in 0..cmp::min(s_distance, e_distance) {
        pos_se >>= 9;
        attacks |= pos_se;
        if occupied & pos_se != 0 {
            break;
        }
    }
    attacks
}