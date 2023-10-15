pub type Bitboard = u64;
pub type Charboard = [[char; 8]; 8];

pub enum Color {
    White,
    Black
}

pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1
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
    let mut res: Vec<Bitboard> = Vec::new();
    while bb != 0 {
        let lsb = bb & bb.wrapping_neg();
        res.push(lsb);
        bb ^= lsb;
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

pub fn print_cb(cb: &Charboard) {
    for (i, row) in cb.iter().enumerate() {
        print!("{} ", 8 - i);
        for &piece in row.iter() {
            if piece == ' ' {
                print!(". ");
                continue;
            }
            else {
                print!("{} ", piece);
            }
        }
        println!();
    }
    println!("  a b c d e f g h");
}