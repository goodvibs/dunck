use crate::board::Board;
use crate::r#move::{CASTLE_FLAG, Move, NO_FLAG, PAWN_DOUBLE_MOVE_FLAG, PROMOTE_TO_BISHOP_FLAG, PROMOTE_TO_KNIGHT_FLAG, PROMOTE_TO_QUEEN_FLAG, PROMOTE_TO_ROOK_FLAG};
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
                let knight_srcs = unpack_bb(self.board.wn);
                for src in knight_srcs {
                    let knight_moves = knight_attacks(src) & !white_occ;
                    for dst in unpack_bb(knight_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), NO_FLAG));
                    }
                }
                let bishop_srcs = unpack_bb(self.board.wb);
                for src in bishop_srcs {
                    let bishop_moves = bishop_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(bishop_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), NO_FLAG));
                    }
                }
                let rook_srcs = unpack_bb(self.board.wr);
                for src in rook_srcs {
                    let rook_moves = rook_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(rook_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), NO_FLAG));
                    }
                }
                let queen_srcs = unpack_bb(self.board.wq);
                for src in queen_srcs {
                    let queen_moves = (rook_attacks(src, all_occ) | bishop_attacks(src, all_occ)) & !white_occ;
                    for dst in unpack_bb(queen_moves) {
                        moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), NO_FLAG));
                    }
                }
                let pawn_srcs = unpack_bb(self.board.wp);
                for src in pawn_srcs.iter() {
                    let single_move_dst = pawn_moves(*src, Color::White) & !all_occ;
                    if single_move_dst == 0 {
                        continue;
                    }
                    if single_move_dst & RANK_3 != 0 {
                        let double_move_dst = pawn_moves(single_move_dst, Color::White) & !all_occ;
                        if double_move_dst != 0 {
                            moves.push(Move::new(src.leading_zeros(), double_move_dst.leading_zeros(), PAWN_DOUBLE_MOVE_FLAG));
                        }
                    }
                    else if single_move_dst & RANK_8 != 0 {
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                        continue;
                    }
                    moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), 0));
                }
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
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), 0));
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
                    moves.push(Move::new(king_src.leading_zeros(), dst.leading_zeros(), NO_FLAG));
                }
                if self.wk_castle && ((white_occ | black_attacks) & WHITE_CASTLE_SHORT == 0) {
                    moves.push(Move::new(king_src.leading_zeros(), (king_src >> 2).leading_zeros(), CASTLE_FLAG));
                }
                if self.wq_castle && ((white_occ | black_attacks) & WHITE_CASTLE_LONG == 0) {
                    moves.push(Move::new(king_src.leading_zeros(), (king_src << 2).leading_zeros(), CASTLE_FLAG));
                }
            },
            Color::Black => {

            }
        }
        moves
    }
}