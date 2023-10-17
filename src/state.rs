use crate::board::Board;
use crate::r#move::*;
use crate::utils::*;
use crate::attacks::*;
use crate::masks::*;

pub struct State {
    pub board: Board,
    pub wk_castle: bool,
    pub wq_castle: bool,
    pub bk_castle: bool,
    pub bq_castle: bool,
    pub in_check: bool,
    pub turn: Color,
    pub ply: u16
}

impl State {
    pub fn initial() -> State {
        State {
            board: Board::initial(),
            wk_castle: true,
            wq_castle: true,
            bk_castle: true,
            bq_castle: true,
            in_check: false,
            turn: Color::White,
            ply: 0
        }
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let white_occ = self.board.white();
        let black_occ = self.board.black();
        let all_occ = white_occ | black_occ;
        let mut moves: Vec<Move> = Vec::new();
        match self.turn {
            Color::White => {
                // knight moves
                let knight_srcs = unpack_bb(self.board.wn);
                for src in knight_srcs {
                    let knight_moves = knight_attacks(src) & !white_occ;
                    for dst in unpack_bb(knight_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), KNIGHT_MOVE_FLAG));
                    }
                }
                // bishop moves
                let bishop_srcs = unpack_bb(self.board.wb);
                for src in bishop_srcs {
                    let bishop_moves = bishop_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(bishop_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), BISHOP_MOVE_FLAG));
                    }
                }
                // rook moves
                let rook_srcs = unpack_bb(self.board.wr);
                for src in rook_srcs {
                    let rook_moves = rook_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(rook_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), ROOK_MOVE_FLAG));
                    }
                }
                // queen moves
                let queen_srcs = unpack_bb(self.board.wq);
                for src in queen_srcs {
                    let queen_moves = (rook_attacks(src, all_occ) | bishop_attacks(src, all_occ)) & !white_occ;
                    for dst in unpack_bb(queen_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), QUEEN_MOVE_FLAG));
                    }
                }
                // pawn pushes
                let pawn_srcs = unpack_bb(self.board.wp);
                for src in pawn_srcs.iter() {
                    let single_move_dst = pawn_moves(*src, Color::White) & !all_occ;
                    if single_move_dst == 0 {
                        continue;
                    }
                    // double moves
                    if single_move_dst & RANK_3 != 0 {
                        let double_move_dst = pawn_moves(single_move_dst, Color::White) & !all_occ;
                        if double_move_dst != 0 {
                            moves.push(Move::new(src.leading_zeros(), double_move_dst.leading_zeros(), PAWN_DOUBLE_MOVE_FLAG));
                        }
                    }
                    else if single_move_dst & RANK_8 != 0 { // promotion
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                        continue;
                    }
                    moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PAWN_MOVE_FLAG));
                }
                // pawn captures
                for src in pawn_srcs {
                    let captures = pawn_attacks(src, Color::White) & black_occ;
                    for dst in unpack_bb(captures) {
                        if dst & RANK_8 != 0 {
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                        }
                        else {
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PAWN_MOVE_FLAG));
                        }
                    }
                }
                let king_src = self.board.wk;
                let mut black_attacks = knight_attacks(self.board.bn) |
                    king_attacks(self.board.bk) |
                    pawn_attacks(self.board.bp, Color::Black);
                for src in unpack_bb(self.board.bb) {
                    black_attacks |= bishop_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.br) {
                    black_attacks |= rook_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.bq) {
                    black_attacks |= bishop_attacks(src, all_occ) | rook_attacks(src, all_occ);
                }
                let king_moves = king_attacks(king_src) & !white_occ & !black_attacks;
                for dst in unpack_bb(king_moves) {
                    moves.push(Move::new(king_src.leading_zeros(), dst.leading_zeros(), KING_MOVE_FLAG));
                }
                if self.wk_castle && ((white_occ | black_attacks) & WHITE_CASTLE_SHORT == 0) {
                    moves.push(Move::new(king_src.leading_zeros(), (king_src >> 2).leading_zeros(), CASTLE_FLAG));
                }
                if self.wq_castle && ((white_occ | black_attacks) & WHITE_CASTLE_LONG == 0) {
                    moves.push(Move::new(king_src.leading_zeros(), (king_src << 2).leading_zeros(), CASTLE_FLAG));
                }
            },
            Color::Black => {
                // TODO: implement
            }
        }
        moves
    }

    pub fn play_move(&mut self, mv: Move) {
        let (src_sq, dst_sq, flag) = mv.unpack();
        let src = 1 << (63 - src_sq);
        let dst = 1 << (63 - dst_sq);
        let src_dst = src | dst;
        match self.turn {
            Color::White => {
                self.board.bp &= !dst;
                self.board.bn &= !dst;
                self.board.bb &= !dst;
                self.board.br &= !dst;
                self.board.bq &= !dst;
                match flag {
                    KNIGHT_MOVE_FLAG => {
                        self.board.wn ^= src_dst;
                    },
                    BISHOP_MOVE_FLAG => {
                        self.board.wb ^= src_dst;
                    },
                    ROOK_MOVE_FLAG => {
                        self.board.wr ^= src_dst;
                    },
                    QUEEN_MOVE_FLAG => {
                        self.board.wq ^= src_dst;
                    },
                    KING_MOVE_FLAG => {
                        self.board.wk ^= src_dst;
                    },
                    PAWN_MOVE_FLAG | PAWN_DOUBLE_MOVE_FLAG | EN_PASSANT_FLAG => {
                        self.board.wp ^= src_dst;
                    },
                    PROMOTE_TO_QUEEN_FLAG => {
                        self.board.wq |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_KNIGHT_FLAG => {
                        self.board.wn |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_ROOK_FLAG => {
                        self.board.wr |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_BISHOP_FLAG => {
                        self.board.wb |= dst;
                        self.board.wp &= !src;
                    },
                    CASTLE_FLAG => {
                        self.board.wk &= !0x08;
                        if dst == src >> 2 { // short castle
                            self.board.wr &= !0x01;
                            self.board.wr |= 0x04;
                            self.board.wk |= 0x02;
                        }
                        else if dst == src << 2 { // long castle
                            self.board.wr &= !0x80;
                            self.board.wr |= 0x10;
                            self.board.wk |= 0x20;
                        }
                    },
                    _ => {
                        panic!("invalid move flag");
                    }
                }
            },
            Color::Black => {
                // TODO: implement
            }
        }
    }
}