use std::fmt;
use crate::consts::*;
use crate::preload::ZOBRIST_TABLE;
use crate::utils::*;
use crate::attacks::*;

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

    pub fn is_in_check(&self, color: Color) -> bool {
        let white_occ = self.white();
        let black_occ = self.black();
        let all_occ = white_occ | black_occ;
        match color {
            Color::White => {
                let mut attacks = pawn_attacks(self.bp, Color::Black);
                attacks |= knight_attacks(self.bn);
                for bb in unpack_bb(self.bb) {
                    attacks |= bishop_attacks(bb, all_occ);
                }
                for bb in unpack_bb(self.br) {
                    attacks |= rook_attacks(bb, all_occ);
                }
                for bb in unpack_bb(self.bq) {
                    attacks |= bishop_attacks(bb, all_occ) | rook_attacks(bb, all_occ);
                }
                attacks |= king_attacks(self.bk);
                attacks & self.wk != 0
            },
            Color::Black => {
                let mut attacks = pawn_attacks(self.wp, Color::White);
                attacks |= knight_attacks(self.wn);
                for bb in unpack_bb(self.wb) {
                    attacks |= bishop_attacks(bb, all_occ);
                }
                for bb in unpack_bb(self.wr) {
                    attacks |= rook_attacks(bb, all_occ);
                }
                for bb in unpack_bb(self.wq) {
                    attacks |= bishop_attacks(bb, all_occ) | rook_attacks(bb, all_occ);
                }
                attacks |= king_attacks(self.wk);
                attacks & self.bk != 0
            }
        }
    }

    pub fn piece_at(&self, square_mask: Bitboard) -> Option<(Piece, Color)> {
        if self.wp & square_mask != 0 {
            Some((Piece::Pawn, Color::White))
        }
        else if self.wn & square_mask != 0 {
            Some((Piece::Knight, Color::White))
        }
        else if self.wb & square_mask != 0 {
            Some((Piece::Bishop, Color::White))
        }
        else if self.wr & square_mask != 0 {
            Some((Piece::Rook, Color::White))
        }
        else if self.wq & square_mask != 0 {
            Some((Piece::Queen, Color::White))
        }
        else if self.wk & square_mask != 0 {
            Some((Piece::King, Color::White))
        }
        else if self.bp & square_mask != 0 {
            Some((Piece::Pawn, Color::Black))
        }
        else if self.bn & square_mask != 0 {
            Some((Piece::Knight, Color::Black))
        }
        else if self.bb & square_mask != 0 {
            Some((Piece::Bishop, Color::Black))
        }
        else if self.br & square_mask != 0 {
            Some((Piece::Rook, Color::Black))
        }
        else if self.bq & square_mask != 0 {
            Some((Piece::Queen, Color::Black))
        }
        else if self.bk & square_mask != 0 {
            Some((Piece::King, Color::Black))
        }
        else {
            None
        }
    }

    pub fn zobrist_hash(&self) -> u64 {
        let mut hash: u64 = 0;
        for index in bb_to_square_indices(self.wp) {
            hash ^= ZOBRIST_TABLE[index as usize][WP];
        }
        for index in bb_to_square_indices(self.wn) {
            hash ^= ZOBRIST_TABLE[index as usize][WN];
        }
        for index in bb_to_square_indices(self.wb) {
            hash ^= ZOBRIST_TABLE[index as usize][WB];
        }
        for index in bb_to_square_indices(self.wr) {
            hash ^= ZOBRIST_TABLE[index as usize][WR];
        }
        for index in bb_to_square_indices(self.wq) {
            hash ^= ZOBRIST_TABLE[index as usize][WQ];
        }
        for index in bb_to_square_indices(self.wk) {
            hash ^= ZOBRIST_TABLE[index as usize][WK];
        }
        for index in bb_to_square_indices(self.bp) {
            hash ^= ZOBRIST_TABLE[index as usize][BP];
        }
        for index in bb_to_square_indices(self.bn) {
            hash ^= ZOBRIST_TABLE[index as usize][BN];
        }
        for index in bb_to_square_indices(self.bb) {
            hash ^= ZOBRIST_TABLE[index as usize][BB];
        }
        for index in bb_to_square_indices(self.br) {
            hash ^= ZOBRIST_TABLE[index as usize][BR];
        }
        for index in bb_to_square_indices(self.bq) {
            hash ^= ZOBRIST_TABLE[index as usize][BQ];
        }
        hash ^= ZOBRIST_TABLE[self.wk.leading_zeros() as usize][WK];
        hash ^= ZOBRIST_TABLE[self.bk.leading_zeros() as usize][BK];
        hash
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