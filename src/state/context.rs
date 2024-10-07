use crate::bitboard::Bitboard;
use crate::masks::{STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::miscellaneous::{Color, ColoredPiece, PieceType};

const fn calc_castling_color_adjustment(color: Color) -> usize {
    (color as usize) << 1
}

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
    
    pub fn handle_capture(&mut self, captured_colored_piece: ColoredPiece, dst_mask: Bitboard) {
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