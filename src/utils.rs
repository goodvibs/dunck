pub type Bitboard = u64;
pub type Charboard = [[char; 8]; 8];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

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

pub fn unpack_bb(mut bb: Bitboard) -> Vec<Bitboard> {
    let mut res: Vec<Bitboard> = Vec::with_capacity(64);
    while bb != 0 {
        let lsb = 1 << bb.trailing_zeros();
        res.push(lsb);
        bb ^= lsb;
    }
    res
}

pub fn bb_to_square_indices(mut bb: Bitboard) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::with_capacity(64);
    while bb != 0 {
        let msb_index = bb.leading_zeros();
        res.push(msb_index as u8);
        bb ^= !0 >> msb_index;
    }
    res
}

pub fn print_bb(bb: Bitboard) {
    for i in 0..8 {
        let shift_amt = 8 * (7 - i);
        println!("{:08b}", (bb & (0xFF << shift_amt)) >> shift_amt);
    }
}

pub fn pprint_bb(bb: Bitboard) {
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