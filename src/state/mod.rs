mod board;
mod context;
mod termination;
mod make_move;
mod movegen;
mod unmake_move;
mod zobrist;
mod fen;

use std::cell::RefCell;
pub use board::*;
pub use context::*;
pub use termination::*;
pub use make_move::*;
pub use movegen::*;
pub use unmake_move::*;
pub use zobrist::*;
pub use fen::*;

use crate::utils::masks::{CASTLING_CHECK_MASK_LONG, CASTLING_CHECK_MASK_SHORT, FILES, RANK_4, STARTING_BK, STARTING_KING_ROOK_GAP_LONG, STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_BR, STARTING_KING_SIDE_WR, STARTING_QUEEN_SIDE_BR, STARTING_QUEEN_SIDE_WR, STARTING_WK};
use crate::utils::*;
use hashbrown::HashMap;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub termination: Option<Termination>,
    pub context: Rc<RefCell<Context>>,
}

impl State {
    pub fn blank() -> State {
        let board = Board::blank();
        let zobrist_hash = board.zobrist_hash;
        State {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            termination: None,
            context: Rc::new(RefCell::new(Context::initial_no_castling(zobrist_hash))),
        }
    }

    pub fn initial() -> State {
        let board = Board::initial();
        let zobrist_hash = board.zobrist_hash;
        State {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            termination: None,
            context: Rc::new(RefCell::new(Context::initial(zobrist_hash))),
        }
    }

    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn assume_and_update_termination(&mut self) {
        self.termination = Some(
            match self.termination {
                Some(termination) => termination,
                None => match self.board.is_color_in_check(self.side_to_move) {
                    true => Termination::Checkmate,
                    false => Termination::Stalemate,
                }
            }
        );
    }

    pub fn has_castling_rights_short(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00001000 >> (color as u8 * 2)) != 0
    }

    pub fn has_castling_rights_long(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00000100 >> (color as u8 * 2)) != 0
    }

    const fn has_castling_space_short(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_SHORT[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    const fn has_castling_space_long(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_LONG[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    fn can_castle_short_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_SHORT[color as usize], color.flip())
    }

    fn can_castle_long_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_LONG[color as usize], color.flip())
    }

    pub fn can_legally_castle_short(&self, color: Color) -> bool {
        self.has_castling_rights_short(color) && self.has_castling_space_short(color) && self.can_castle_short_without_check(color)
    }

    pub fn can_legally_castle_long(&self, color: Color) -> bool {
        self.has_castling_rights_long(color) && self.has_castling_space_long(color) && self.can_castle_long_without_check(color)
    }

    /// Rigorous check for whether the state is valid.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.board.is_valid() &&
            self.has_valid_side_to_move() &&
            self.has_valid_castling_rights() &&
            self.has_valid_double_pawn_push() &&
            self.has_valid_halfmove_clock() &&
            self.is_not_in_illegal_check() &&
            self.is_zobrist_consistent()
    }

    /// Quick check for whether the state is probably valid, should be used after making pseudo-legal moves.
    pub fn is_probably_valid(&self) -> bool {
        self.board.has_valid_kings() && self.is_not_in_illegal_check()
    }
    
    pub fn is_zobrist_consistent(&self) -> bool {
        self.board.zobrist_hash == self.context.borrow().zobrist_hash
    }

    pub fn is_not_in_illegal_check(&self) -> bool {
        !self.board.is_color_in_check(self.side_to_move.flip())
    }

    pub fn has_valid_halfmove_clock(&self) -> bool {
        let context = self.context.borrow();
        context.has_valid_halfmove_clock() && context.halfmove_clock as u16 <= self.halfmove
    }

    pub fn has_valid_side_to_move(&self) -> bool {
        self.halfmove % 2 == self.side_to_move as u16
    }

    pub fn has_valid_castling_rights(&self) -> bool {
        let context = self.context.borrow();

        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let rooks_bb = self.board.piece_type_masks[PieceType::Rook as usize];

        let white_bb = self.board.color_masks[Color::White as usize];
        let black_bb = self.board.color_masks[Color::Black as usize];

        let is_white_king_in_place = (kings_bb & white_bb & STARTING_WK) != 0;
        let is_black_king_in_place = (kings_bb & black_bb & STARTING_BK) != 0;

        if !is_white_king_in_place && context.castling_rights & 0b00001100 != 0 {
            return false;
        }

        if !is_black_king_in_place && context.castling_rights & 0b00000011 != 0 {
            return false;
        }

        let is_white_king_side_rook_in_place = (rooks_bb & white_bb & STARTING_KING_SIDE_WR) != 0;
        if !is_white_king_side_rook_in_place && (context.castling_rights & 0b00001000) != 0 {
            return false;
        }

        let is_white_queen_side_rook_in_place = (rooks_bb & white_bb & STARTING_QUEEN_SIDE_WR) != 0;
        if !is_white_queen_side_rook_in_place && (context.castling_rights & 0b00000100) != 0 {
            return false;
        }

        let is_black_king_side_rook_in_place = (rooks_bb & black_bb & STARTING_KING_SIDE_BR) != 0;
        if !is_black_king_side_rook_in_place && (context.castling_rights & 0b00000010) != 0 {
            return false;
        }

        let is_black_queen_side_rook_in_place = (rooks_bb & black_bb & STARTING_QUEEN_SIDE_BR) != 0;
        if !is_black_queen_side_rook_in_place && (context.castling_rights & 0b00000001) != 0 {
            return false;
        }

        true
    }

    pub fn has_valid_double_pawn_push(&self) -> bool {
        match self.context.borrow().double_pawn_push {
            -1 => true,
            file if file > 7 || file < -1 => false,
            file => {
                if self.halfmove < 1 {
                    return false;
                }
                let color_just_moved = self.side_to_move.flip();
                let pawns_bb = self.board.piece_type_masks[PieceType::Pawn as usize];
                let colored_pawns_bb = pawns_bb & self.board.color_masks[color_just_moved as usize];
                let file_mask = FILES[file as usize];
                let rank_mask = RANK_4 << (color_just_moved as Bitboard * 8); // 4 for white, 5 for black
                colored_pawns_bb & file_mask & rank_mask != 0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // 
    // #[test]
    // fn test_state_has_valid_side_to_move() {
    //     let state = State::blank();
    //     assert!(state.has_valid_side_to_move());
    // 
    //     let mut state = State::initial();
    //     assert!(state.has_valid_side_to_move());
    //     state.side_to_move = Color::Black;
    //     assert!(!state.has_valid_side_to_move());
    // 
    //     state.halfmove = 99;
    //     assert!(state.has_valid_side_to_move());
    //     state.halfmove = 100;
    //     assert!(!state.has_valid_side_to_move());
    // }
    // 
    // #[test]
    // fn test_state_has_valid_castling_rights() {
    //     let state = State::blank();
    //     assert!(state.has_valid_castling_rights());
    // 
    //     let mut state = State::initial();
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.context.castling_rights = 0b00000000;
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.context.castling_rights = 0b00001111;
    // 
    //     state.board.clear_piece_at(STARTING_WK);
    //     assert!(!state.has_valid_castling_rights());
    // 
    //     state.board.put_colored_piece_at(ColoredPiece::WhiteKing, STARTING_WK);
    //     state.board.clear_piece_at(STARTING_KING_SIDE_BR);
    //     assert!(state.board.is_valid());
    //     assert!(!state.has_valid_castling_rights());
    //     state.context.castling_rights = 0b00001101;
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.board.put_colored_piece_at(ColoredPiece::WhiteRook, STARTING_KING_SIDE_WR);
    //     state.board.clear_piece_at(STARTING_QUEEN_SIDE_WR);
    //     assert!(state.board.is_valid());
    //     assert!(!state.has_valid_castling_rights());
    // 
    //     state.board.put_colored_piece_at(ColoredPiece::WhiteRook, STARTING_QUEEN_SIDE_WR);
    //     state.board.clear_piece_at(STARTING_BK);
    //     assert!(!state.has_valid_castling_rights());
    //     state.board.put_colored_piece_at(ColoredPiece::BlackKing, Square::E4.to_mask());
    //     assert!(state.board.is_valid());
    //     assert!(!state.has_valid_castling_rights());
    //     let castling_info = state.context.castling_rights;
    //     state.context.castling_rights &= !0b00000011;
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.context.castling_rights = castling_info;
    //     state.board.clear_piece_at(Square::E4.to_mask());
    //     state.board.put_colored_piece_at(ColoredPiece::BlackKing, STARTING_BK);
    //     state.board.clear_piece_at(STARTING_KING_SIDE_BR);
    //     assert!(state.board.is_valid());
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.board.put_colored_piece_at(ColoredPiece::BlackRook, STARTING_KING_SIDE_BR);
    //     state.board.clear_piece_at(STARTING_QUEEN_SIDE_BR);
    //     assert!(!state.has_valid_castling_rights());
    // 
    //     state.board.put_colored_piece_at(ColoredPiece::BlackRook, STARTING_QUEEN_SIDE_BR);
    //     assert!(state.has_valid_castling_rights());
    // 
    //     state.context.castling_rights = 0b00000010;
    //     assert!(state.has_valid_castling_rights());
    // }
    // 
    // #[test]
    // fn test_state_has_valid_double_pawn_push() {
    //     let state = State::blank();
    //     assert!(state.has_valid_double_pawn_push());
    // 
    //     let state = State::initial();
    //     assert_eq!(state.context.double_pawn_push, -1);
    //     assert!(state.has_valid_double_pawn_push());
    // 
    //     // todo
    // }
}