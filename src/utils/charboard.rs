use crate::utils::bitboard::Bitboard;
use crate::state::Board;
use crate::utils::Square;

pub type Charboard = [[char; 8]; 8];

pub const EMPTY_CHARBOARD: Charboard = [[' '; 8]; 8];

pub const INITIAL_CHARBOARD: Charboard = [
    ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
    ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
    [' '; 8],
    [' '; 8],
    [' '; 8],
    [' '; 8],
    ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
    ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R']
];

pub const INITIAL_CHARBOARD_PRETTY: Charboard = [
    ['♜', '♞', '♝', '♛', '♚', '♝', '♞', '♜'],
    ['♟', '♟', '♟', '♟', '♟', '♟', '♟', '♟'],
    [' '; 8],
    [' '; 8],
    [' '; 8],
    [' '; 8],
    ['♙', '♙', '♙', '♙', '♙', '♙', '♙', '♙'],
    ['♖', '♘', '♗', '♕', '♔', '♗', '♘', '♖']
];

pub const SQUARE_NAMES: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1"
];

pub const COLORED_PIECE_CHARS: [char; 12] = [
    'P', 'N', 'B', 'R', 'Q', 'K',
    'p', 'n', 'b', 'r', 'q', 'k'
];

pub const COLORED_PIECE_CHARS_PRETTY: [char; 12] = [
    '♙', '♘', '♗', '♖', '♕', '♔',
    '♟', '♞', '♝', '♜', '♛', '♚'
];

pub fn cb_to_bb(cb: &Charboard) -> Bitboard {
    let mut bb: Bitboard = 0;
    for i in 0..8 {
        for j in 0..8 {
            if cb[i][j] != ' ' {
                bb |= 1 << (63 - (i * 8 + j));
            }
        }
    }
    bb
}

pub fn bb_to_cb(mut bb: Bitboard) -> Charboard {
    let mut cb: Charboard = [[' '; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            if bb & 1 != 0 {
                cb[7 - i][7 - j] = 'X';
            }
            bb >>= 1;
        }
    }
    cb
}

pub fn print_bb(bb: Bitboard) {
    for i in 0..8 {
        let shift_amt = 8 * (7 - i);
        println!("{:08b}", (bb & (0xFF << shift_amt)) >> shift_amt);
    }
}

pub fn print_bb_pretty(bb: Bitboard) {
    print_cb(&bb_to_cb(bb));
}

pub fn cb_to_string(cb: &Charboard) -> String {
    let mut res = String::new();
    for i in 0..8u8 {
        res += &*format!("{} ", 8 - i);
        for j in 0..8u8 {
            if cb[i as usize][j as usize] == ' ' {
                res.push('.');
            }
            else {
                res.push(cb[i as usize][j as usize])
            }
            res.push(' ');
        }
        res.push('\n')
    }
    res + "  a b c d e f g h"
}

pub fn print_cb(cb: &Charboard) {
    println!("{}", cb_to_string(cb));
}

impl Board {
    pub fn to_cb(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::iter_all().enumerate() {
            let colored_piece = self.get_colored_piece_at(*square);
            cb[i / 8][i % 8] = colored_piece.to_char();
        }
        cb
    }

    pub fn to_cb_pretty(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for (i, square) in Square::iter_all().enumerate() {
            let colored_piece = self.get_colored_piece_at(*square);
            cb[i / 8][i % 8] = colored_piece.to_char_pretty();
        }
        cb
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", cb_to_string(&self.to_cb_pretty()).as_str())
    }
}