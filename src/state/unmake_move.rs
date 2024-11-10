//! Contains the implementation of the `State::unmake_move` method.

use std::cell::RefCell;
use std::rc::Rc;
use crate::r#move::{Move, MoveFlag};
use crate::state::{Context, State, Termination};
use crate::utils::{Bitboard, Color, ColoredPiece, PieceType, Square};
use crate::utils::masks::{STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};

impl State {
    fn unprocess_promotion(&mut self, dst_square: Square, src_square: Square, promotion: PieceType) {
        self.board.remove_piece_type_at(promotion, dst_square); // remove promoted piece
        self.board.put_piece_type_at(PieceType::Pawn, src_square); // put pawn back

        self.unprocess_possible_capture(dst_square); // add possible captured piece back
    }

    fn unprocess_normal(&mut self, dst_square: Square, src_square: Square) {
        let moved_piece = self.board.get_piece_type_at(dst_square); // get moved piece
        self.board.move_piece_type(moved_piece, src_square, dst_square); // move piece back

        self.unprocess_possible_capture(dst_square); // add possible captured piece back
    }

    fn unprocess_possible_capture(&mut self, dst_square: Square) {
        // remove captured piece and get captured piece type
        let captured_piece = self.context.borrow().captured_piece;
        if captured_piece != PieceType::NoPieceType {
            // piece was captured
            self.board.put_color_at(self.side_to_move, dst_square); // put captured color back
            self.board.put_piece_type_at(captured_piece, dst_square); // put captured piece back
        }
    }

    fn unprocess_en_passant(&mut self, dst_square: Square, src_square: Square) {
        let en_passant_capture_square = match self.side_to_move {
            Color::White => unsafe { Square::from(dst_square as u8 - 8) },
            Color::Black => unsafe { Square::from(dst_square as u8 + 8) }
        };
        
        self.board.move_piece_type(PieceType::Pawn, src_square, dst_square); // move pawn back
        self.board.put_color_at(self.side_to_move, en_passant_capture_square); // put captured color back
        self.board.put_piece_type_at(PieceType::Pawn, en_passant_capture_square); // put captured piece back
    }

    fn unprocess_castling(&mut self, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.get_mask();

        self.board.move_piece_type(PieceType::King, src_square, dst_square); // move king back

        let is_king_side = dst_mask & STARTING_KING_ROOK_GAP_SHORT[self.side_to_move.flip() as usize] != 0;

        let rook_src_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 3) },
            false => unsafe { Square::from(src_square as u8 - 4) }
        };
        let rook_dst_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 1) },
            false => unsafe { Square::from(src_square as u8 - 1) }
        };

        self.board.move_colored_piece(ColoredPiece::from(self.side_to_move.flip(), PieceType::Rook), rook_src_square, rook_dst_square); // move rook back
    }

    /// Undoes a move from State without checking if it is valid, legal, or even applied to the current position.
    /// This method is used to undo a move that was previously made with `State::make_move`, regardless of
    /// whether the move was legal. However, the move must have been valid (not malformed).
    pub fn unmake_move(&mut self, mv: Move) {
        let (dst_square, src_square, promotion, flag) = mv.unpack();

        self.board.move_color(self.side_to_move.flip(), src_square, dst_square);

        match flag {
            MoveFlag::NormalMove => self.unprocess_normal(dst_square, src_square),
            MoveFlag::Promotion => self.unprocess_promotion(dst_square, src_square, promotion),
            MoveFlag::EnPassant => self.unprocess_en_passant(dst_square, src_square),
            MoveFlag::Castling => self.unprocess_castling(dst_square, src_square)
        }
        
        // update data members
        self.halfmove -= 1;
        self.side_to_move = self.side_to_move.flip();
        let old_context = self.context.borrow().previous.as_ref().expect("No previous context").clone();
        self.context = old_context;
        self.termination = None;
    }
}