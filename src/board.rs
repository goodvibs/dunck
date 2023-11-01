use std::fmt;
use crate::utils::*;

#[derive(Clone)]
pub struct Board {
    pub wp: Bitboard,
    pub wn: Bitboard,
    pub wb: Bitboard,
    pub wr: Bitboard,
    pub wq: Bitboard,
    pub wk: Bitboard,
    pub bp: Bitboard,
    pub bn: Bitboard,
    pub bb: Bitboard,
    pub br: Bitboard,
    pub bq: Bitboard,
    pub bk: Bitboard
}

impl Board {
    pub fn initial() -> Board {
        Board {
            wp: 0x000000000000FF00,
            wn: 0x0000000000000042,
            wb: 0x0000000000000024,
            wr: 0x0000000000000081,
            wq: 0x0000000000000010,
            wk: 0x0000000000000008,
            bp: 0x00FF000000000000,
            bn: 0x4200000000000000,
            bb: 0x2400000000000000,
            br: 0x8100000000000000,
            bq: 0x1000000000000000,
            bk: 0x0800000000000000
        }
    }

    pub fn blank() -> Board {
        Board {
            wp: 0,
            wn: 0,
            wb: 0,
            wr: 0,
            wq: 0,
            wk: 0,
            bp: 0,
            bn: 0,
            bb: 0,
            br: 0,
            bq: 0,
            bk: 0
        }
    }

    pub fn from_cb(cb: Charboard) -> Board {
        let mut board = Board::blank();
        for (i, row) in cb.iter().enumerate() {
            for (j, &piece) in row.iter().enumerate() {
                let loc = 1 << (63 - i * 8 - j);
                match piece {
                    'P' => board.wp |= loc,
                    'N' => board.wn |= loc,
                    'B' => board.wb |= loc,
                    'R' => board.wr |= loc,
                    'Q' => board.wq |= loc,
                    'K' => board.wk |= loc,
                    'p' => board.bp |= loc,
                    'n' => board.bn |= loc,
                    'b' => board.bb |= loc,
                    'r' => board.br |= loc,
                    'q' => board.bq |= loc,
                    'k' => board.bk |= loc,
                    _ => ()
                }
            }
        }
        board
    }

    pub fn to_cb(&self) -> Charboard {
        let mut cb = [[' '; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                let mask = 1 << (63 - (i * 8 + j));
                if self.wp & mask != 0 {
                    cb[i][j] = 'P';
                }
                else if self.wn & mask != 0 {
                    cb[i][j] = 'N';
                }
                else if self.wb & mask != 0 {
                    cb[i][j] = 'B';
                }
                else if self.wr & mask != 0 {
                    cb[i][j] = 'R';
                }
                else if self.wq & mask != 0 {
                    cb[i][j] = 'Q';
                }
                else if self.wk & mask != 0 {
                    cb[i][j] = 'K';
                }
                else if self.bp & mask != 0 {
                    cb[i][j] = 'p';
                }
                else if self.bn & mask != 0 {
                    cb[i][j] = 'n';
                }
                else if self.bb & mask != 0 {
                    cb[i][j] = 'b';
                }
                else if self.br & mask != 0 {
                    cb[i][j] = 'r';
                }
                else if self.bq & mask != 0 {
                    cb[i][j] = 'q';
                }
                else if self.bk & mask != 0 {
                    cb[i][j] = 'k';
                }
            }
        }
        cb
    }

    pub fn white(&self) -> Bitboard {
        self.wp | self.wn | self.wb | self.wr | self.wq | self.wk
    }

    pub fn black(&self) -> Bitboard {
        self.bp | self.bn | self.bb | self.br | self.bq | self.bk
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", cb_to_string(&self.to_cb()).as_str())
    }
}