use crate::miscellaneous::PieceType;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Context {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: PieceType,
    pub previous: Option<Box<Context>>
}

impl Context {
    pub fn new(halfmove_clock: u8, double_pawn_push: i8, castling_info: u8, captured_piece: PieceType, previous: Option<Box<Context>>) -> Context {
        Context {
            halfmove_clock,
            double_pawn_push,
            castling_rights: castling_info,
            captured_piece,
            previous
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
}