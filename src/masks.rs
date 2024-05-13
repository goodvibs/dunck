use crate::bitboard::Bitboard;

pub const FILE_A: Bitboard = 0x8080808080808080;
pub const FILE_B: Bitboard = 0x4040404040404040;
pub const FILE_C: Bitboard = 0x2020202020202020;
pub const FILE_D: Bitboard = 0x1010101010101010;
pub const FILE_E: Bitboard = 0x0808080808080808;
pub const FILE_F: Bitboard = 0x0404040404040404;
pub const FILE_G: Bitboard = 0x0202020202020202;
pub const FILE_H: Bitboard = 0x0101010101010101;
pub const FILES_AB: Bitboard = FILE_A | FILE_B;
pub const FILES_GH: Bitboard = FILE_G | FILE_H;

pub const RANK_1: Bitboard = 0x00000000000000FF;
pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
pub const RANK_4: Bitboard = 0x00000000FF000000;
pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
pub const RANK_7: Bitboard = 0x00FF000000000000;
pub const RANK_8: Bitboard = 0xFF00000000000000;

pub const OUTER_EDGES: Bitboard = FILE_A | FILE_H | RANK_1 | RANK_8;
pub const DARK_SQUARES: Bitboard = 0x55AA55AA55AA55AA;
pub const LIGHT_SQUARES: Bitboard = !DARK_SQUARES;
pub const CENTER_4: Bitboard = 0x0000001818000000;
pub const CENTER_16: Bitboard = 0x00003C3C3C3C0000;
pub const WHITE_CASTLE_SHORT: Bitboard = 0x0000000000000006;
pub const WHITE_CASTLE_LONG: Bitboard = 0x0000000000000070;
pub const BLACK_CASTLE_SHORT: Bitboard = 0x0600000000000000;
pub const BLACK_CASTLE_LONG: Bitboard = 0x7000000000000000;

pub const FILES: [Bitboard; 8] = [
    FILE_A,
    FILE_B,
    FILE_C,
    FILE_D,
    FILE_E,
    FILE_F,
    FILE_G,
    FILE_H
];

pub const RANKS: [Bitboard; 8] = [
    RANK_1,
    RANK_2,
    RANK_3,
    RANK_4,
    RANK_5,
    RANK_6,
    RANK_7,
    RANK_8
];

pub const DIAGONALS: [Bitboard; 15] = [ // /// from top left to bottom right
    0x8000000000000000,
    0x4080000000000000,
    0x2040800000000000,
    0x1020408000000000,
    0x0810204080000000,
    0x0408102040800000,
    0x0204081020408000,
    0x0102040810204080,
    0x0001020408102040,
    0x0000010204081020,
    0x0000000102040810,
    0x0000000001020408,
    0x0000000000010204,
    0x0000000000000102,
    0x0000000000000001
];

pub const ANTIDIAGONALS: [Bitboard; 15] = [ // \\\ from bottom left to top right
    0x0000000000000080,
    0x0000000000008040,
    0x0000000000804020,
    0x0000000080402010,
    0x0000008040201008,
    0x0000804020100804,
    0x0080402010080402,
    0x8040201008040201,
    0x4020100804020100,
    0x2010080402010000,
    0x1008040201000000,
    0x0804020100000000,
    0x0402010000000000,
    0x0201000000000000,
    0x0100000000000000
];

pub const STARTING_WP: Bitboard = 0x000000000000FF00;
pub const STARTING_WN: Bitboard = 0x0000000000000042;
pub const STARTING_WB: Bitboard = 0x0000000000000024;
pub const STARTING_WR: Bitboard = 0x0000000000000081;
pub const STARTING_WQ: Bitboard = 0x0000000000000010;
pub const STARTING_WK: Bitboard = 0x0000000000000008;
pub const STARTING_BP: Bitboard = 0x00FF000000000000;
pub const STARTING_BN: Bitboard = 0x4200000000000000;
pub const STARTING_BB: Bitboard = 0x2400000000000000;
pub const STARTING_BR: Bitboard = 0x8100000000000000;
pub const STARTING_BQ: Bitboard = 0x1000000000000000;
pub const STARTING_BK: Bitboard = 0x0800000000000000;

pub const STARTING_WHITE: Bitboard = STARTING_WP | STARTING_WN | STARTING_WB | STARTING_WR | STARTING_WQ | STARTING_WK;
pub const STARTING_BLACK: Bitboard = STARTING_BP | STARTING_BN | STARTING_BB | STARTING_BR | STARTING_BQ | STARTING_BK;
pub const STARTING_ALL: Bitboard = STARTING_WHITE | STARTING_BLACK;

pub const STARTING_WR_SHORT: Bitboard = 0x0000000000000001;
pub const STARTING_WR_LONG: Bitboard = 0x0000000000000080;
pub const STARTING_BR_SHORT: Bitboard = 0x0100000000000000;
pub const STARTING_BR_LONG: Bitboard = 0x8000000000000000;