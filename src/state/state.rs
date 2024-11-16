//! Contains the State struct, which is the main struct for representing a position in a chess game.

use std::cell::RefCell;
use std::rc::Rc;
use crate::state::{Board, Context, Termination};
use crate::utils::{Bitboard, Color, PieceType};
use crate::utils::masks::{CASTLING_CHECK_MASK_LONG, CASTLING_CHECK_MASK_SHORT, FILES, RANK_4, STARTING_BK, STARTING_KING_ROOK_GAP_LONG, STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_BR, STARTING_KING_SIDE_WR, STARTING_QUEEN_SIDE_BR, STARTING_QUEEN_SIDE_WR, STARTING_WK};

/// A struct containing all the information needed to represent a position in a chess game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub termination: Option<Termination>,
    pub context: Rc<RefCell<Context>>,
}

impl State {
    /// Creates a blank state with no pieces on the board.
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

    /// Creates an initial state with the standard starting position.
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

    /// Gets the fullmove number of the position.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    /// Assumes the game has ended and updates the termination as checkmate or stalemate.
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
    
    /// Checks if the game has ended and updates the termination as checkmate or stalemate.
    pub fn check_and_update_termination(&mut self) {
        if self.calc_legal_moves().is_empty() {
            self.assume_and_update_termination();
        }
    }

    /// Returns whether the current side to move has short castling rights.
    pub fn has_castling_rights_short(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00001000 >> (color as u8 * 2)) != 0
    }

    /// Returns whether the current side to move has long castling rights.
    pub fn has_castling_rights_long(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00000100 >> (color as u8 * 2)) != 0
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for short castling.
    /// Else, returns false.
    const fn has_castling_space_short(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_SHORT[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for long castling.
    /// Else, returns false.
    const fn has_castling_space_long(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_LONG[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    /// Returns true if the opponent has no pieces that can attack the squares the king moves through for short castling.
    /// Else, returns false.
    fn can_castle_short_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_SHORT[color as usize], color.flip())
    }

    /// Returns true if the opponent has no pieces that can attack the squares the king moves through for long castling.
    /// Else, returns false.
    fn can_castle_long_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_LONG[color as usize], color.flip())
    }

    /// Returns true if the current side to move can legally castle short.
    /// Else, returns false.
    pub fn can_legally_castle_short(&self, color: Color) -> bool {
        self.has_castling_rights_short(color) && self.has_castling_space_short(color) && self.can_castle_short_without_check(color)
    }

    /// Returns true if the current side to move can legally castle long.
    /// Else, returns false.
    pub fn can_legally_castle_long(&self, color: Color) -> bool {
        self.has_castling_rights_long(color) && self.has_castling_space_long(color) && self.can_castle_long_without_check(color)
    }
    
    /// Rigorous check for whether the current positional information is consistent and valid.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.board.is_unequivocally_valid() &&
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

    /// Checks if the zobrist hash in the board is consistent with the zobrist hash in the context.
    pub fn is_zobrist_consistent(&self) -> bool {
        self.board.zobrist_hash == self.context.borrow().zobrist_hash
    }

    /// Returns true if the opponent king is not in check.
    /// Else, returns false.
    pub fn is_not_in_illegal_check(&self) -> bool {
        !self.board.is_color_in_check(self.side_to_move.flip())
    }

    /// Checks if the halfmove clock is valid and consistent with the halfmove counter.
    pub fn has_valid_halfmove_clock(&self) -> bool {
        let context = self.context.borrow();
        context.has_valid_halfmove_clock() && context.halfmove_clock as u16 <= self.halfmove
    }

    /// Checks if the side to move is consistent with the halfmove counter.
    pub fn has_valid_side_to_move(&self) -> bool {
        self.halfmove % 2 == self.side_to_move as u16
    }

    /// Checks if the castling rights are consistent with the position of the rooks and kings.
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

    /// Checks if the double pawn push is consistent with the position of the pawns.
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