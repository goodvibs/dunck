//! Context struct and methods

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Bitboard;
use crate::utils::masks::{STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::utils::{Color, ColoredPiece, PieceType, Square};

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Context {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: PieceType,
    pub previous: Option<Rc<RefCell<Context>>>,
    pub zobrist_hash: Bitboard
}

impl Context {
    /// Creates a new context linking to the previous context
    pub fn new_from(previous_context: Rc<RefCell<Context>>, zobrist_hash: Bitboard) -> Context {
        let previous = previous_context.borrow();
        Context {
            halfmove_clock: previous.halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous.castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: Some(previous_context.clone()),
            zobrist_hash
        }
    }

    /// Creates a new context with no previous context.
    /// Castling rights are set to full.
    /// This is used for the initial position.
    pub fn initial(zobrist_hash: Bitboard) -> Context {
        Context {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00001111,
            captured_piece: PieceType::NoPieceType,
            previous: None,
            zobrist_hash
        }
    }

    /// Creates a new context with no previous context.
    /// Castling rights are set to none.
    pub fn initial_no_castling(zobrist_hash: Bitboard) -> Context {
        Context {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00000000,
            captured_piece: PieceType::NoPieceType,
            previous: None,
            zobrist_hash
        }
    }

    /// Checks if the halfmove clock is valid (less than or equal to 100).
    pub fn has_valid_halfmove_clock(&self) -> bool {
        self.halfmove_clock <= 100
    }
    
    /// Gets the last context belonging to a position that could be the same as the current position
    /// (same side to move, nonzero halfmove clock), if it exists.
    /// Else, returns None.
    /// This essentially gets the context of the position two halfmoves ago, if it exists and there
    /// was no halfmove_clock reset in between.
    pub fn get_previous_possible_repetition(&self) -> Option<Rc<RefCell<Context>>> {
        match &self.previous {
            Some(previous) => {
                if previous.borrow().halfmove_clock == 0 {
                    return None;
                }
                match &previous.borrow().previous {
                    Some(previous_previous) => Some(previous_previous.clone()),
                    None => None
                }
            },
            None => None
        }
    }
    
    /// Checks if threefold repetition has occurred by checking if the zobrist hash of the current
    /// position has occurred three times, searching backward until the halfmove clock indicates
    /// that no more possible repetitions could have occurred, or until there are no more previous
    /// contexts.
    pub fn has_threefold_repetition_occurred(&self) -> bool {
        if self.halfmove_clock < 4 {
            return false;
        }

        let mut count = 1;
        
        let mut current_context = self.get_previous_possible_repetition();
        let mut expected_halfmove_clock = self.halfmove_clock - 2;
        
        while let Some(context) = current_context {
            let context = context.borrow();
            
            if context.halfmove_clock != expected_halfmove_clock {
                break;
            }
            
            if context.zobrist_hash == self.zobrist_hash {
                count += 1;
                if count == 3 {
                    return true;
                }
            }
            
            expected_halfmove_clock = expected_halfmove_clock.wrapping_sub(2);
            current_context = context.get_previous_possible_repetition();
        }
        
        false
    }
}