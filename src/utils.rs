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