use crate::utils::Bitboard;
use crate::utils::masks::{STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::utils::{Color, ColoredPiece, PieceType, Square};

const fn calc_castling_color_adjustment(color: Color) -> usize {
    (color as usize) << 1
}

const fn is_double_pawn_push(dst_square: Square, src_square: Square) -> bool {
    let dst_mask = dst_square.to_mask();
    let src_mask = src_square.to_mask();
    
    dst_mask & (src_mask << 16) != 0 || dst_mask & (src_mask >> 16) != 0
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
    
    pub fn new_from(previous_context: Box<Context>) -> Context {
        Context {
            halfmove_clock: previous_context.halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous_context.castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: Some(previous_context)
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
    
    pub fn handle_promotion_disregarding_capture(&mut self) {
        self.halfmove_clock = 0;
    }
    
    pub fn handle_normal_disregarding_capture(&mut self, moved_piece: ColoredPiece, dst_square: Square, src_square: Square) {
        let moved_piece_type = moved_piece.get_piece_type();
        let moved_piece_color = moved_piece.get_color();

        match moved_piece_type {
            PieceType::Pawn => self.handle_normal_pawn_move_disregarding_capture(dst_square, src_square),
            PieceType::King => self.handle_normal_king_move_disregarding_capture(moved_piece_color),
            PieceType::Rook => self.handle_normal_rook_move_disregarding_capture(moved_piece_color, src_square),
            _ => {}
        }
    }
    
    fn handle_normal_pawn_move_disregarding_capture(&mut self, dst_square: Square, src_square: Square) {
        self.halfmove_clock = 0;
        if is_double_pawn_push(dst_square, src_square) {
            self.double_pawn_push = (src_square as u8 % 8) as i8;
        }
    }
    
    fn handle_normal_king_move_disregarding_capture(&mut self, moved_piece_color: Color) {
        let castling_color_adjustment = calc_castling_color_adjustment(moved_piece_color);
        self.castling_rights &= !(0b00001100 >> castling_color_adjustment);
    }
    
    fn handle_normal_rook_move_disregarding_capture(&mut self, moved_piece_color: Color, src_square: Square) {
        let src_mask = src_square.to_mask();
        let castling_color_adjustment = calc_castling_color_adjustment(moved_piece_color);
        
        let is_king_side = src_mask & (1u64 << (moved_piece_color as u64 * 7 * 8));
        let is_queen_side = src_mask & (0b10000000u64 << (moved_piece_color as u64 * 7 * 8));
        let king_side_mask = (is_king_side != 0) as u8 * (0b00001000 >> castling_color_adjustment);
        let queen_side_mask = (is_queen_side != 0) as u8 * (0b00000100 >> castling_color_adjustment);
        
        self.castling_rights &= !(king_side_mask | queen_side_mask);
    }
    
    pub fn handle_en_passant(&mut self) {
        self.halfmove_clock = 0;
        self.captured_piece = PieceType::Pawn;
    }
    
    pub fn handle_castle(&mut self, color: Color) {
        let right_shift = calc_castling_color_adjustment(color) as u8;
        self.halfmove_clock = 0;
        self.castling_rights &= !(0b00001100 >> right_shift);
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