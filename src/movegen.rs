use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, pawn_moves, rook_attacks};
use crate::bitboard::unpack_bb;
use crate::charboard::print_bb;
use crate::miscellaneous::{Color, PieceType, Square};
use crate::masks::{STARTING_BK_BR_GAP_LONG, STARTING_BK_BR_GAP_SHORT, FILE_A, RANK_3, RANK_4, RANK_5, RANK_6, RANK_8, STARTING_WK_WR_GAP_LONG, STARTING_WK_WR_GAP_SHORT, RANK_1};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;

fn add_pawn_promotion_moves(moves: &mut Vec<Move>, src: Square, dst: Square) {
    for promotion_piece in PieceType::iter_between(PieceType::Knight, PieceType::Queen) {
        moves.push(Move::new(dst, src, promotion_piece, MoveFlag::Promotion));
    }
}

impl State {
    fn add_normal_pawn_captures_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: &Vec<u64>) {
        let opposite_color = self.side_to_move.flip();
        let opposite_color_bb = self.board.bb_by_color[opposite_color as usize];

        let promotion_rank = match self.side_to_move {
            Color::White => RANK_8,
            Color::Black => RANK_1
        };

        for src in pawn_srcs.clone() {
            let captures = pawn_attacks(src, self.side_to_move) & opposite_color_bb;
            for dst in unpack_bb(captures) {
                let move_src = unsafe { Square::from(src.leading_zeros() as u8) };
                let move_dst = unsafe { Square::from(dst.leading_zeros() as u8) };
                if dst & promotion_rank != 0 {
                    add_pawn_promotion_moves(moves, move_src, move_dst);
                }
                else {
                    moves.push(Move::new_non_promotion(move_dst, move_src, MoveFlag::NormalMove));
                }
            }
        }
    }

    fn add_en_passant_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let pawns_bb = self.board.bb_by_piece_type[PieceType::Pawn as usize] & same_color_bb;

        let (src_rank_bb, dst_rank_bb) = match self.side_to_move {
            Color::White => (RANK_5, RANK_6),
            Color::Black => (RANK_4, RANK_3),
        };

        if self.context.double_pawn_push != -1 { // if en passant is possible
            for &direction in [-1, 1].iter() { // left and right
                let double_pawn_push_file = self.context.double_pawn_push as i32 + direction;
                if double_pawn_push_file >= 0 && double_pawn_push_file <= 7 { // if within bounds
                    let double_pawn_push_file_mask = FILE_A >> double_pawn_push_file;
                    if pawns_bb & double_pawn_push_file_mask & src_rank_bb != 0 {
                        let move_src = unsafe { Square::from(src_rank_bb.leading_zeros() as u8 + double_pawn_push_file as u8) };
                        let move_dst = unsafe { Square::from(dst_rank_bb.leading_zeros() as u8 + self.context.double_pawn_push as u8) };
                        moves.push(Move::new_non_promotion(move_dst, move_src, MoveFlag::EnPassant));
                    }
                }
            }
        }
    }
    
    fn add_pawn_push_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: &Vec<u64>) {
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let promotion_rank = RANK_8 >> (self.side_to_move as u8 * 7 * 8); // RANK_8 for white, RANK_1 for black

        // pawn pushes
        let single_push_rank = match self.side_to_move {
            Color::White => RANK_3,
            Color::Black => RANK_6
        };
        for src_bb in pawn_srcs.iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };

            // single moves
            let single_move_dst = pawn_moves(*src_bb, self.side_to_move) & !all_occupancy_bb;
            if single_move_dst == 0 { // if no single moves
                continue;
            }

            let single_move_dst_square = unsafe { Square::from(single_move_dst.leading_zeros() as u8) };

            // double push
            if single_move_dst & single_push_rank != 0 {
                let double_move_dst = pawn_moves(single_move_dst, self.side_to_move) & !all_occupancy_bb;
                if double_move_dst != 0 {
                    unsafe {
                        let double_move_dst_square = Square::from(double_move_dst.leading_zeros() as u8);
                        moves.push(Move::new_non_promotion(double_move_dst_square, src_square, MoveFlag::NormalMove));
                    }
                }
            }
            else if single_move_dst & promotion_rank != 0 { // promotion
                add_pawn_promotion_moves(moves, src_square, single_move_dst_square);
                continue;
            }

            // single push (non-promotion)
            moves.push(Move::new_non_promotion(single_move_dst_square, src_square, MoveFlag::NormalMove));
        }
    }
    
    fn add_all_pawn_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let pawns_bb = self.board.bb_by_piece_type[PieceType::Pawn as usize] & same_color_bb;
        let pawn_srcs = unpack_bb(pawns_bb);

        self.add_normal_pawn_captures_pseudolegal(moves, &pawn_srcs);
        self.add_en_passant_pseudolegal(moves);
        self.add_pawn_push_pseudolegal(moves, &pawn_srcs);
    }

    fn add_knight_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];

        let knights_bb = self.board.bb_by_piece_type[PieceType::Knight as usize] & same_color_bb;
        for src_bb in unpack_bb(knights_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let knight_moves = knight_attacks(*src_bb) & !same_color_bb;
            for dst_bb in unpack_bb(knight_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_bishop_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let bishops_bb = self.board.bb_by_piece_type[PieceType::Bishop as usize] & same_color_bb;
        for src_bb in unpack_bb(bishops_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let bishop_moves = bishop_attacks(*src_bb, all_occupancy_bb) & !same_color_bb;
            for dst_bb in unpack_bb(bishop_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_rook_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let rooks_bb = self.board.bb_by_piece_type[PieceType::Rook as usize] & same_color_bb;
        for src_bb in unpack_bb(rooks_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let rook_moves = rook_attacks(*src_bb, all_occupancy_bb) & !same_color_bb;
            for dst_bb in unpack_bb(rook_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_queen_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let queens_bb = self.board.bb_by_piece_type[PieceType::Queen as usize] & same_color_bb;
        for src_bb in unpack_bb(queens_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let queen_moves = (rook_attacks(*src_bb, all_occupancy_bb) | bishop_attacks(*src_bb, all_occupancy_bb)) & !same_color_bb;
            for dst_bb in unpack_bb(queen_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new_non_promotion(dst_square, src_square, MoveFlag::NormalMove));
            }
        }
    }

    fn add_king_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        // king moves
        let king_src_bb = self.board.bb_by_piece_type[PieceType::King as usize] & same_color_bb;
        let king_src_square = unsafe { Square::from(king_src_bb.leading_zeros() as u8) };
        let king_moves = king_attacks(king_src_bb) & !same_color_bb;
        for dst_bb in unpack_bb(king_moves).iter() {
            let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
            moves.push(Move::new_non_promotion(dst_square, king_src_square, MoveFlag::NormalMove));
        }
    }
    
    fn add_castling_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let king_src_square = match self.side_to_move {
            Color::White => Square::E1,
            Color::Black => Square::E8
        };

        if self.can_legally_castle_short(self.side_to_move) {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 + 2) };
            moves.push(Move::new_non_promotion(king_dst_square, king_src_square, MoveFlag::Castling));
        }
        if self.can_legally_castle_long(self.side_to_move) {
            let king_dst_square = unsafe { Square::from(king_src_square as u8 - 2) };
            moves.push(Move::new_non_promotion(king_dst_square, king_src_square, MoveFlag::Castling));
        }
    }

    pub fn get_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        self.add_all_pawn_pseudolegal(&mut moves);
        self.add_knight_pseudolegal(&mut moves);
        self.add_bishop_pseudolegal(&mut moves);
        self.add_rook_pseudolegal(&mut moves);
        self.add_queen_pseudolegal(&mut moves);
        self.add_king_pseudolegal(&mut moves);
        self.add_castling_pseudolegal(&mut moves);

        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::miscellaneous::ColoredPiece;

    #[test]
    fn test_pawn_normal_captures_pseudolegal() {
        // let mut state = State::initial();
        // state.board.set_piece_at(Square::D4, ColoredPiece::WhitePawn);
        // state.board.set_piece_at(Square::E5, ColoredPiece::BlackPawn);
        // let moves = state.get_pseudolegal_moves();
        // assert_eq!(moves.len(), 1);
        // assert_eq!(moves[0], Move::new(Square::D4, Square::E5, MoveFlag::PawnMove));
    }
}