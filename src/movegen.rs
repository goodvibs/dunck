use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, pawn_moves, rook_attacks};
use crate::bitboard::unpack_bb;
use crate::enums::{Color, PieceType, Square};
use crate::masks::{FILE_A, RANK_3, RANK_5, RANK_6, RANK_8};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;

fn add_pawn_promotion_moves(moves: &mut Vec<Move>, src: Square, dst: Square) {
    moves.push(Move::new(src, dst, MoveFlag::PromoteToQueen));
    moves.push(Move::new(src, dst, MoveFlag::PromoteToKnight));
    moves.push(Move::new(src, dst, MoveFlag::PromoteToRook));
    moves.push(Move::new(src, dst, MoveFlag::PromoteToBishop));
}

impl State {
    fn add_normal_pawn_captures_pseudolegal(&self, moves: &mut Vec<Move>, pawn_srcs: &Vec<u64>) {
        let opposite_color = self.side_to_move.flip();
        let opposite_color_bb = self.board.bb_by_color[opposite_color as usize];

        let promotion_rank = RANK_8 >> (self.side_to_move as u8 * 7 * 8); // RANK_8 for white, RANK_1 for black

        for src in pawn_srcs.clone() {
            let captures = pawn_attacks(src, self.side_to_move) & opposite_color_bb;
            for dst in unpack_bb(captures) {
                let move_src = unsafe { Square::from(src.leading_zeros() as u8) };
                let move_dst = unsafe { Square::from(dst.leading_zeros() as u8) };
                if dst & promotion_rank != 0 {
                    add_pawn_promotion_moves(moves, move_src, move_dst);
                }
                else {
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PawnMove));
                }
            }
        }
    }
    
    fn add_en_passant_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let pawns_bb = self.board.bb_by_piece_type[PieceType::Pawn as usize] & same_color_bb;

        let (src_offset, dst_offset) = match self.side_to_move {
            Color::White => (24, 16),
            Color::Black => (32, 40)
        };
        if (*self.context).double_pawn_push != -1 { // if en passant is possible
            for direction in [-1, 1].iter() { // left and right
                let double_pawn_push_file = self.context.double_pawn_push as i32 + direction;
                if double_pawn_push_file >= 0 && double_pawn_push_file <= 7 { // if within bounds
                    let double_pawn_push_file_mask = FILE_A >> double_pawn_push_file;
                    if pawns_bb & double_pawn_push_file_mask & RANK_5 != 0 {
                        let move_src = unsafe { Square::from((src_offset + double_pawn_push_file) as u8) };
                        let move_dst = unsafe { Square::from((dst_offset + self.context.double_pawn_push) as u8) };
                        moves.push(Move::new(move_src, move_dst, MoveFlag::EnPassant));
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
                        moves.push(Move::new(src_square, double_move_dst_square, MoveFlag::PawnDoubleMove));
                    }
                }
            }
            else if single_move_dst & promotion_rank != 0 { // promotion
                add_pawn_promotion_moves(moves, src_square, single_move_dst_square);
                continue;
            }

            // single push
            moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PawnMove));
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
                moves.push(Move::new(src_square, dst_square, MoveFlag::KnightMove));
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
                moves.push(Move::new(src_square, dst_square, MoveFlag::BishopMove));
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
                moves.push(Move::new(src_square, dst_square, MoveFlag::RookMove));
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
                moves.push(Move::new(src_square, dst_square, MoveFlag::QueenMove));
            }
        }
    }

    fn add_king_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        // king moves
        let king_src_bb = self.board.bb_by_piece_type[PieceType::King as usize] & same_color_bb;
        let king_src_square = unsafe { Square::from(king_src_bb.leading_zeros() as u8) };
        let king_moves = king_attacks(king_src_bb) & !all_occupancy_bb;
        for dst_bb in unpack_bb(king_moves).iter() {
            let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
            moves.push(Move::new(king_src_square, dst_square, MoveFlag::KingMove));
        }
    }
    
    fn add_castle_pseudolegal(&self, moves: &mut Vec<Move>) {
        let same_color_bb = self.board.bb_by_color[self.side_to_move as usize];
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        let (king_src_square, king_dst_square, rook_src_square, rook_dst_square) = match self.side_to_move {
            Color::White => (Square::E1, Square::G1, Square::H1, Square::F1),
            Color::Black => (Square::E8, Square::G8, Square::H8, Square::F8)
        };

        let king_src_bb = 1 << (63 - king_src_square as u8);
        let rook_src_bb = 1 << (63 - rook_src_square as u8);

        if self.context.castling_info & (0b00001000 >> (self.side_to_move as usize * 2)) != 0 { // king side
            let king_side_empty = king_src_bb | (1 << (63 - Square::F1 as u8)) | (1 << (63 - Square::G1 as u8)) & !all_occupancy_bb == 0;
            if king_side_empty {
                moves.push(Move::new(king_src_square, king_dst_square, MoveFlag::Castle));
            }
        }
        if self.context.castling_info & (0b00000100 >> (self.side_to_move as usize * 2)) != 0 { // queen side
            let queen_side_empty = king_src_bb | (1 << (63 - Square::D1 as u8)) | (1 << (63 - Square::C1 as u8)) | (1 << (63 - Square::B1 as u8)) & !all_occupancy_bb == 0;
            if queen_side_empty {
                moves.push(Move::new(king_src_square, king_dst_square, MoveFlag::Castle));
            }
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

        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::ColoredPiece;

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