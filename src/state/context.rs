use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Bitboard;
use crate::utils::masks::{STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::utils::{Color, ColoredPiece, PieceType, Square};

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Context {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: PieceType,
    pub previous: Option<Rc<RefCell<Context>>>
}

impl Context {
    pub fn new(halfmove_clock: u8, double_pawn_push: i8, castling_info: u8, captured_piece: PieceType, previous: Option<Rc<RefCell<Context>>>) -> Context {
        Context {
            halfmove_clock,
            double_pawn_push,
            castling_rights: castling_info,
            captured_piece,
            previous
        }
    }
    
    pub fn new_from(previous_context: Rc<RefCell<Context>>) -> Context {
        let previous = previous_context.borrow();
        Context {
            halfmove_clock: previous.halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous.castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: Some(previous_context.clone())
        }
    }

    pub fn initial() -> Context {
        Context {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00001111,
            captured_piece: PieceType::NoPieceType,
            previous: None
        }
    }

    pub fn initial_no_castling() -> Context {
        Context {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00000000,
            captured_piece: PieceType::NoPieceType,
            previous: None
        }
    }

    pub fn has_valid_halfmove_clock(&self) -> bool {
        self.halfmove_clock <= 100
    }
}