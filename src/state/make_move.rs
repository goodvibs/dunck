use crate::bitboard::Bitboard;
use crate::masks::{STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::miscellaneous::{Color, ColoredPiece, PieceType, Square};
use crate::r#move::Move;
use crate::r#move::move_flag::MoveFlag;
use crate::state::State;
use crate::state::context::Context;
use crate::state::termination::Termination;
use crate::state::zobrist::get_piece_zobrist_hash;

const fn calc_castling_color_adjustment(color: Color) -> usize {
    (color as usize) << 1
}

impl State {
    fn handle_promotion(&mut self, dst_square: Square, src_square: Square, promotion: PieceType, new_context: &mut Context) {
        self.handle_possible_capture(dst_square, new_context);
        
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        
        self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src_mask;
        self.board.bb_by_piece_type[promotion as usize] |= dst_mask;
        
        new_context.handle_promotion_disregarding_capture();
    }
    
    fn handle_normal(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        self.handle_possible_capture(dst_square, new_context);
        
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        let castling_color_adjustment = calc_castling_color_adjustment(self.side_to_move);
        
        let moved_piece = self.board.get_piece_type_at(src_mask);

        self.board.bb_by_piece_type[moved_piece as usize] &= !src_mask;
        self.board.bb_by_piece_type[moved_piece as usize] |= dst_mask;

        new_context.handle_normal_disregarding_capture(ColoredPiece::from(self.side_to_move, moved_piece), dst_square, src_square);
    }

    fn handle_possible_capture(&mut self, dst_square: Square, new_context: &mut Context) {
        let dst_mask = dst_square.to_mask();
        let opposite_color = self.side_to_move.flip();
        let castling_color_adjustment = calc_castling_color_adjustment(self.side_to_move);
        
        self.board.bb_by_color[opposite_color as usize] &= !dst_mask; // clear opposite color piece presence

        // remove captured piece and get captured piece type
        let captured_piece = self.board.remove_and_get_captured_piece_type_at(dst_mask);
        if captured_piece != PieceType::NoPieceType {
            new_context.handle_capture(ColoredPiece::from(opposite_color, captured_piece), dst_mask);
            self.zobrist_hash ^= get_piece_zobrist_hash(dst_square, captured_piece);
        }
    }
    
    fn handle_en_passant(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        let opposite_color = self.side_to_move.flip();
        
        let en_passant_capture = ((dst_mask << 8) * self.side_to_move as u64) | ((dst_mask >> 8) * opposite_color as u64);
        self.board.bb_by_piece_type[PieceType::Pawn as usize] ^= dst_mask | src_mask | en_passant_capture;
        self.board.bb_by_color[opposite_color as usize] &= !en_passant_capture;
        
        new_context.handle_en_passant();
    }
    
    fn handle_castling(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();

        self.board.bb_by_piece_type[PieceType::King as usize] ^= dst_mask | src_mask;

        let is_king_side = dst_mask & STARTING_KING_ROOK_GAP_SHORT[self.side_to_move as usize] != 0;

        let rook_src_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 3) },
            false => unsafe { Square::from(src_square as u8 - 4) }
        };
        let rook_dst_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 1) },
            false => unsafe { Square::from(src_square as u8 - 1) }
        };
        let rook_src_dst = rook_src_square.to_mask() | rook_dst_square.to_mask();

        self.board.bb_by_color[self.side_to_move as usize] ^= rook_src_dst;
        self.board.bb_by_piece_type[PieceType::Rook as usize] ^= rook_src_dst;

        new_context.handle_castle(self.side_to_move);
    }
    
    pub fn make_move(&mut self, mv: Move) { // todo: split into smaller functions for unit testing
        let (dst_square, src_square, promotion, flag) = mv.unpack();
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();

        let mut new_context = Context::new(
            self.context.halfmove_clock + 1,
            -1,
            self.context.castling_rights.clone(),
            PieceType::NoPieceType,
            Some(self.context.clone())
        );

        let castling_color_adjustment = calc_castling_color_adjustment(self.side_to_move);
        let opposite_color = self.side_to_move.flip();
        let previous_castling_rights = self.context.castling_rights.clone();

        self.board.bb_by_color[self.side_to_move as usize] ^= dst_mask | src_mask; // sufficient for all moves except the rook in castling

        match flag {
            MoveFlag::NormalMove => self.handle_normal(dst_square, src_square, &mut new_context),
            MoveFlag::Promotion => self.handle_promotion(dst_square, src_square, promotion, &mut new_context),
            MoveFlag::EnPassant => self.handle_en_passant(dst_square, src_square, &mut new_context),
            MoveFlag::Castling => self.handle_castling(dst_square, src_square, &mut new_context)
        }

        // update data members
        self.halfmove += 1;
        self.side_to_move = opposite_color;
        self.context = Box::new(new_context);
        self.in_check = self.board.is_color_in_check(self.side_to_move);
        self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize] = self.board.bb_by_color[Color::White as usize] | self.board.bb_by_color[Color::Black as usize];

        if self.board.are_both_sides_insufficient_material() {
            self.termination = Some(Termination::InsufficientMaterial);
        }
        else if self.context.halfmove_clock == 100 { // fifty move rule
            self.termination = Some(Termination::FiftyMoveRule);
        }
        else {
            // update Zobrist table
            let position_count = self.increment_position_count();

            // check for repetition
            if position_count == 3 {
                self.termination = Some(Termination::ThreefoldRepetition);
            }
        }
    }
}