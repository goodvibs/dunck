//! Contains the implementation of the `State::make_move` method.

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::masks::{STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::utils::{Bitboard, Color, ColoredPiece, PieceType, Square};
use crate::r#move::{Move, MoveFlag};
use crate::state::context::Context;
use crate::state::termination::Termination;
use crate::state::zobrist::get_piece_zobrist_hash;
use crate::state::State;

impl State {
    fn process_promotion(&mut self, dst_square: Square, src_square: Square, promotion: PieceType, new_context: &mut Context) {
        self.process_possible_capture(dst_square, new_context);
        
        self.board.remove_piece_type_at(PieceType::Pawn, src_square);
        self.board.put_piece_type_at(promotion, dst_square);
        
        new_context.process_promotion_disregarding_capture();
    }
    
    fn process_normal(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        self.process_possible_capture(dst_square, new_context);
        
        let moved_piece = self.board.get_piece_type_at(src_square);
        assert_ne!(moved_piece, PieceType::NoPieceType);
        self.board.move_piece_type(moved_piece, dst_square, src_square);
        new_context.process_normal_disregarding_capture(ColoredPiece::from(self.side_to_move, moved_piece), dst_square, src_square);
    }

    fn process_possible_capture(&mut self, dst_square: Square, new_context: &mut Context) {
        let dst_mask = dst_square.get_mask();
        let opposite_color = self.side_to_move.flip();
        
        self.board.remove_color_at(opposite_color, dst_square);

        // remove captured piece and get captured piece type
        let captured_piece = self.board.get_piece_type_at(dst_square);
        if captured_piece != PieceType::NoPieceType {
            self.board.remove_piece_type_at(captured_piece, dst_square);
            new_context.process_capture(ColoredPiece::from(opposite_color, captured_piece), dst_mask);
        }
    }
    
    fn process_en_passant(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        let opposite_color = self.side_to_move.flip();
        
        let en_passant_capture_square = match opposite_color {
            Color::White => unsafe { Square::from(dst_square as u8 - 8) },
            Color::Black => unsafe { Square::from(dst_square as u8 + 8) }
        };

        self.board.remove_color_at(opposite_color, en_passant_capture_square);
        self.board.move_piece_type(PieceType::Pawn, dst_square, src_square);
        self.board.remove_piece_type_at(PieceType::Pawn, en_passant_capture_square);
        
        new_context.process_en_passant();
    }
    
    fn process_castling(&mut self, dst_square: Square, src_square: Square, new_context: &mut Context) {
        let dst_mask = dst_square.get_mask();

        self.board.move_piece_type(PieceType::King, dst_square, src_square);

        let is_king_side = dst_mask & STARTING_KING_ROOK_GAP_SHORT[self.side_to_move as usize] != 0;

        let rook_src_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 3) },
            false => unsafe { Square::from(src_square as u8 - 4) }
        };
        let rook_dst_square = match is_king_side {
            true => unsafe { Square::from(src_square as u8 + 1) },
            false => unsafe { Square::from(src_square as u8 - 1) }
        };

        self.board.move_colored_piece(ColoredPiece::from(self.side_to_move, PieceType::Rook), rook_dst_square, rook_src_square);

        new_context.process_castling(self.side_to_move);
    }
    
    /// Applies a move without checking if it is valid or legal.
    /// All make_move calls with valid (not malformed) moves
    /// should be fully able to be undone by unmake_move.
    pub fn make_move(&mut self, mv: Move) {
        let (dst_square, src_square, promotion, flag) = mv.unpack();

        let mut new_context = Context::new_from(Rc::clone(&self.context), 0);

        self.board.move_color(self.side_to_move, dst_square, src_square);

        match flag {
            MoveFlag::NormalMove => self.process_normal(dst_square, src_square, &mut new_context),
            MoveFlag::Promotion => self.process_promotion(dst_square, src_square, promotion, &mut new_context),
            MoveFlag::EnPassant => self.process_en_passant(dst_square, src_square, &mut new_context),
            MoveFlag::Castling => self.process_castling(dst_square, src_square, &mut new_context)
        }

        new_context.zobrist_hash = self.board.zobrist_hash;
        
        // update data members
        self.halfmove += 1;
        self.side_to_move = self.side_to_move.flip();
        self.context = Rc::new(RefCell::new(new_context));

        if self.board.are_both_sides_insufficient_material(true) {
            self.termination = Some(Termination::InsufficientMaterial);
        }
        else if self.context.borrow().halfmove_clock == 100 { // fifty move rule
            self.termination = Some(Termination::FiftyMoveRule);
        }
        else if self.context.borrow().has_threefold_repetition_occurred() {
            // check for repetition
            self.termination = Some(Termination::ThreefoldRepetition);
        }
    }
}

impl Context {
    fn process_promotion_disregarding_capture(&mut self) {
        self.halfmove_clock = 0;
    }

    fn process_normal_disregarding_capture(&mut self, moved_piece: ColoredPiece, dst_square: Square, src_square: Square) {
        let moved_piece_type = moved_piece.get_piece_type();
        let moved_piece_color = moved_piece.get_color();

        match moved_piece_type {
            PieceType::Pawn => self.process_normal_pawn_move_disregarding_capture(dst_square, src_square),
            PieceType::King => self.process_normal_king_move_disregarding_capture(moved_piece_color),
            PieceType::Rook => self.process_normal_rook_move_disregarding_capture(moved_piece_color, src_square),
            _ => {}
        }
    }

    fn process_normal_pawn_move_disregarding_capture(&mut self, dst_square: Square, src_square: Square) {
        self.halfmove_clock = 0;
        if is_double_pawn_push(dst_square, src_square) {
            self.double_pawn_push = (src_square as u8 % 8) as i8;
        }
    }

    fn process_normal_king_move_disregarding_capture(&mut self, moved_piece_color: Color) {
        let castling_color_adjustment = calc_castling_color_adjustment(moved_piece_color);
        self.castling_rights &= !(0b00001100 >> castling_color_adjustment);
    }

    fn process_normal_rook_move_disregarding_capture(&mut self, moved_piece_color: Color, src_square: Square) {
        let src_mask = src_square.get_mask();
        let castling_color_adjustment = calc_castling_color_adjustment(moved_piece_color);

        let is_king_side = src_mask & (1u64 << (moved_piece_color as u64 * 7 * 8));
        let is_queen_side = src_mask & (0b10000000u64 << (moved_piece_color as u64 * 7 * 8));
        let king_side_mask = (is_king_side != 0) as u8 * (0b00001000 >> castling_color_adjustment);
        let queen_side_mask = (is_queen_side != 0) as u8 * (0b00000100 >> castling_color_adjustment);

        self.castling_rights &= !(king_side_mask | queen_side_mask);
    }

    fn process_en_passant(&mut self) {
        self.halfmove_clock = 0;
        self.captured_piece = PieceType::Pawn;
    }

    fn process_castling(&mut self, color: Color) {
        let right_shift = calc_castling_color_adjustment(color) as u8;
        self.halfmove_clock = 0;
        self.castling_rights &= !(0b00001100 >> right_shift);
    }

    fn process_capture(&mut self, captured_colored_piece: ColoredPiece, dst_mask: Bitboard) {
        let captured_color = captured_colored_piece.get_color();
        let captured_piece = captured_colored_piece.get_piece_type();

        self.captured_piece = captured_piece;
        self.halfmove_clock = 0;
        if captured_piece == PieceType::Rook {
            let king_side_rook_mask = STARTING_KING_SIDE_ROOK[captured_color as usize];
            let queen_side_rook_mask = STARTING_QUEEN_SIDE_ROOK[captured_color as usize];
            let right_shift = calc_castling_color_adjustment(captured_color) as u8;
            if dst_mask & king_side_rook_mask != 0 {
                self.castling_rights &= !(0b00001000 >> right_shift);
            }
            else if dst_mask & queen_side_rook_mask != 0 {
                self.castling_rights &= !(0b00000100 >> right_shift);
            }
        }
    }
}

const fn calc_castling_color_adjustment(color: Color) -> usize {
    (color as usize) << 1
}

const fn is_double_pawn_push(dst_square: Square, src_square: Square) -> bool {
    let dst_mask = dst_square.get_mask();
    let src_mask = src_square.get_mask();

    dst_mask & (src_mask << 16) != 0 || dst_mask & (src_mask >> 16) != 0
}