use std::collections::HashMap;
use crate::board::Board;
use crate::charboard::print_bb_pretty;
use crate::r#move::*;
use crate::miscellaneous::*;
use crate::masks::{CASTLING_CHECK_MASK_LONG, CASTLING_CHECK_MASK_SHORT, STARTING_KING_ROOK_GAP_LONG, STARTING_KING_ROOK_GAP_SHORT, FILES, RANK_4, STARTING_BK, STARTING_QUEEN_SIDE_BR, STARTING_KING_SIDE_BR, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK, STARTING_WK, STARTING_QUEEN_SIDE_WR, STARTING_KING_SIDE_WR};
use crate::pgn::pgn_move_tree::PgnParseError;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Termination {
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule
}

impl Termination {
    pub fn is_decisive(&self) -> bool {
        self == &Termination::Checkmate
    }

    pub fn is_draw(&self) -> bool {
        !self.is_decisive()
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct StateContext {
    // copied from previous
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq
    
    // recomputed on every move
    pub captured_piece: PieceType,
    pub previous: Option<Box<StateContext>>
}

impl StateContext {
    pub fn new(halfmove_clock: u8, double_pawn_push: i8, castling_info: u8, captured_piece: PieceType, previous: Option<Box<StateContext>>) -> StateContext {
        StateContext {
            halfmove_clock,
            double_pawn_push,
            castling_rights: castling_info,
            captured_piece,
            previous
        }
    }
    
    pub fn initial() -> StateContext {
        StateContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00001111,
            captured_piece: PieceType::NoPieceType,
            previous: None
        }
    }
    
    pub fn initial_no_castling() -> StateContext {
        StateContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00000000,
            captured_piece: PieceType::NoPieceType,
            previous: None
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub board: Board,
    pub in_check: bool,
    pub position_counts: HashMap<u64, u8>,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub termination: Option<Termination>,
    pub context: Box<StateContext>
}

impl State {
    pub fn blank() -> State {
        let position_count: HashMap<u64, u8> = HashMap::new();
        State {
            board: Board::blank(),
            in_check: false,
            position_counts: position_count,
            side_to_move: Color::White,
            halfmove: 0,
            termination: None,
            context: Box::new(StateContext::initial_no_castling())
        }
    }

    pub fn initial() -> State {
        let board = Board::initial();
        let position_count: HashMap<u64, u8> = HashMap::from([(board.zobrist_hash(), 1)]);
        State {
            board,
            in_check: false,
            position_counts: position_count,
            side_to_move: Color::White,
            halfmove: 0,
            termination: None,
            context: Box::new(StateContext::initial())
        }
    }

    pub fn from_pgn(pgn: &str) -> Result<State, PgnParseError> {
        // let move_tree = PgnMoveTree::from_pgn(pgn);
        todo!()
    }

    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub const fn has_castling_rights_short(&self, color: Color) -> bool {
        self.context.castling_rights & (0b00001000 >> (color as u8 * 2)) != 0
    }

    pub const fn has_castling_rights_long(&self, color: Color) -> bool {
        self.context.castling_rights & (0b00000100 >> (color as u8 * 2)) != 0
    }

    const fn has_castling_space_short(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_SHORT[color as usize] & self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize] == 0
    }

    const fn has_castling_space_long(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_LONG[color as usize] & self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize] == 0
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
    
    pub fn get_legal_moves(&self) -> Vec<Move> { // todo: filter out illegal moves
        let pseudolegal_moves = self.get_pseudolegal_moves();
        let mut filtered_moves = Vec::new();
        for move_ in pseudolegal_moves {
            let mut new_state = self.clone();
            new_state.make_move(move_);
            if new_state.is_valid() && !new_state.board.is_color_in_check(self.side_to_move) {
                filtered_moves.push(move_);
            }
        }
        filtered_moves
    }
    
    pub fn make_move(&mut self, mv: Move) { // todo: split into smaller functions for unit testing
        let (dst_square, src_square, promotion, flag) = mv.unpack();
        let dst = 1 << (63 - dst_square as u8);
        let src = 1 << (63 - src_square as u8);
        let src_dst = src | dst;
        
        let mut new_context = StateContext::new(
            self.context.halfmove_clock + 1,
            -1,
            self.context.castling_rights.clone(),
            PieceType::NoPieceType,
            Some(self.context.clone())
        );
        
        let castling_color_adjustment = self.side_to_move as usize * 2;
        let opposite_color = self.side_to_move.flip();
        let previous_castling_rights = self.context.castling_rights.clone();
        
        self.board.bb_by_color[self.side_to_move as usize] ^= src_dst; // sufficient for all moves except the rook in castling
        
        match flag {
            MoveFlag::NormalMove => {
                self.board.bb_by_color[opposite_color as usize] &= !dst; // clear opponent's piece if any
                let captured_piece = self.board.process_uncolored_capture_and_get_captured_piece_type_at(dst);
                if captured_piece != PieceType::NoPieceType {
                    new_context.captured_piece = captured_piece;
                    new_context.halfmove_clock = 0;
                    if captured_piece == PieceType::Rook {
                        let king_side_rook_mask = STARTING_KING_SIDE_ROOK[opposite_color as usize];
                        let queen_side_rook_mask = STARTING_QUEEN_SIDE_ROOK[opposite_color as usize];
                        let right_shift: u8 = match opposite_color {
                            Color::White => 0,
                            Color::Black => 2
                        };
                        if dst & king_side_rook_mask != 0 {
                            new_context.castling_rights &= !(0b00001000 >> right_shift);
                        }
                        else if dst & queen_side_rook_mask != 0 {
                            new_context.castling_rights &= !(0b00000100 >> right_shift);
                        }
                    }
                }
                
                let moved_piece = self.board.get_piece_type_at(src);
                
                self.board.bb_by_piece_type[moved_piece as usize] &= !src;
                self.board.bb_by_piece_type[moved_piece as usize] |= dst;
                
                match moved_piece {
                    PieceType::Pawn => {
                        new_context.halfmove_clock = 0;
                        if dst & (src << 16) != 0 || dst & (src >> 16) != 0 { // double pawn push
                            new_context.double_pawn_push = (src_square as u8 % 8) as i8;
                        }
                    },
                    PieceType::King => {
                        new_context.castling_rights &= !0b00001100 >> castling_color_adjustment;
                    },
                    PieceType::Rook => {
                        let is_king_side = src & (1u64 << (self.side_to_move as u64 * 7 * 8));
                        let is_queen_side = src & (0b10000000u64 << (self.side_to_move as u64 * 7 * 8));
                        let king_side_mask = (is_king_side != 0) as u8 * (0b00001000 >> castling_color_adjustment);
                        let queen_side_mask = (is_queen_side != 0) as u8 * (0b00000100 >> castling_color_adjustment);
                        new_context.castling_rights &= !(king_side_mask | queen_side_mask);
                    },
                    _ => {}
                }
            },
            MoveFlag::EnPassant => { // en passant capture
                let en_passant_capture = ((dst << 8) * self.side_to_move as u64) | ((dst >> 8) * opposite_color as u64);
                self.board.bb_by_piece_type[PieceType::Pawn as usize] ^= src_dst | en_passant_capture;
                self.board.bb_by_color[opposite_color as usize] &= !en_passant_capture;
                new_context.captured_piece = PieceType::Pawn;
                new_context.halfmove_clock = 0;
            },
            MoveFlag::Castling => { // src is king's origin square, dst is king's destination square
                new_context.castling_rights &= !0b00001100 >> castling_color_adjustment;

                self.board.bb_by_piece_type[PieceType::King as usize] ^= src_dst;
                
                let is_king_side = dst & STARTING_KING_ROOK_GAP_SHORT[self.side_to_move as usize] != 0;

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
            },
            MoveFlag::Promotion => {
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                for piece_type_int in PieceType::Pawn as usize..PieceType::LIMIT {
                    self.board.bb_by_piece_type[piece_type_int] &= !dst;
                }
                self.board.bb_by_color[opposite_color as usize] &= !dst;
                self.board.bb_by_piece_type[promotion as usize] |= dst;
            }
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
    
    pub fn unmake_move(&mut self, mv: Move) {
        // todo
    }
    
    pub fn is_valid(&self) -> bool {
        self.board.is_valid() &&
        self.has_valid_side_to_move() &&
        self.has_valid_castling_rights() &&
        self.has_valid_double_pawn_push() &&
        self.has_valid_halfmove_clock()
    }
    
    pub fn has_valid_halfmove_clock(&self) -> bool {
        self.context.halfmove_clock <= 100 && self.context.halfmove_clock as u16 <= self.halfmove
    }
    
    pub fn has_valid_side_to_move(&self) -> bool {
        self.halfmove % 2 == self.side_to_move as u16
    }
    
    pub fn has_valid_castling_rights(&self) -> bool {
        let kings_bb = self.board.bb_by_piece_type[PieceType::King as usize];
        let rooks_bb = self.board.bb_by_piece_type[PieceType::Rook as usize];
        
        let white_bb = self.board.bb_by_color[Color::White as usize];
        let black_bb = self.board.bb_by_color[Color::Black as usize];
        
        let is_white_king_in_place = (kings_bb & white_bb & STARTING_WK) != 0;
        let is_black_king_in_place = (kings_bb & black_bb & STARTING_BK) != 0;
        
        if !is_white_king_in_place && self.context.castling_rights & 0b00001100 != 0 {
            return false;
        }
        
        if !is_black_king_in_place && self.context.castling_rights & 0b00000011 != 0 {
            return false;
        }
        
        let is_white_king_side_rook_in_place = (rooks_bb & white_bb & STARTING_KING_SIDE_WR) != 0;
        if !is_white_king_side_rook_in_place && (self.context.castling_rights & 0b00001000) != 0 {
            return false;
        }
        
        let is_white_queen_side_rook_in_place = (rooks_bb & white_bb & STARTING_QUEEN_SIDE_WR) != 0;
        if !is_white_queen_side_rook_in_place && (self.context.castling_rights & 0b00000100) != 0 {
            return false;
        }
        
        let is_black_king_side_rook_in_place = (rooks_bb & black_bb & STARTING_KING_SIDE_BR) != 0;
        if !is_black_king_side_rook_in_place && (self.context.castling_rights & 0b00000010) != 0 {
            return false;
        }
        
        let is_black_queen_side_rook_in_place = (rooks_bb & black_bb & STARTING_QUEEN_SIDE_BR) != 0;
        if !is_black_queen_side_rook_in_place && (self.context.castling_rights & 0b00000001) != 0 {
            return false;
        }
        
        true
    }
    
    pub fn has_valid_double_pawn_push(&self) -> bool {
        match self.context.double_pawn_push {
            -1 => true,
            file if file > 7 || file < -1 => false,
            file => {
                if self.halfmove < 1 {
                    return false;
                }
                let color_just_moved = self.side_to_move.flip();
                let pawns_bb = self.board.bb_by_piece_type[PieceType::Pawn as usize];
                let colored_pawns_bb = pawns_bb & self.board.bb_by_color[color_just_moved as usize];
                let file_mask = FILES[file as usize];
                let rank_mask = RANK_4 << (color_just_moved as u64 * 8); // 4 for white, 5 for black
                colored_pawns_bb & file_mask & rank_mask != 0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_has_valid_side_to_move() {
        let state = State::blank();
        assert!(state.has_valid_side_to_move());
        
        let mut state = State::initial();
        assert!(state.has_valid_side_to_move());
        state.side_to_move = Color::Black;
        assert!(!state.has_valid_side_to_move());

        state.halfmove = 99;
        assert!(state.has_valid_side_to_move());
        state.halfmove = 100;
        assert!(!state.has_valid_side_to_move());
    }
    
    #[test]
    fn test_state_has_valid_castling_rights() {
        let state = State::blank();
        assert!(state.has_valid_castling_rights());
        
        let mut state = State::initial();
        assert!(state.has_valid_castling_rights());
        
        state.context.castling_rights = 0b00000000;
        assert!(state.has_valid_castling_rights());
        
        state.context.castling_rights = 0b00001111;
        
        state.board.clear_pieces_at(STARTING_WK);
        assert!(!state.has_valid_castling_rights());
        
        state.board.put_colored_pieces_at(ColoredPiece::WhiteKing, STARTING_WK);
        state.board.clear_pieces_at(STARTING_KING_SIDE_BR);
        assert!(state.board.is_valid());
        assert!(!state.has_valid_castling_rights());
        state.context.castling_rights = 0b00001101;
        assert!(state.has_valid_castling_rights());
        
        state.board.put_colored_pieces_at(ColoredPiece::WhiteRook, STARTING_KING_SIDE_WR);
        state.board.clear_pieces_at(STARTING_QUEEN_SIDE_WR);
        assert!(state.board.is_valid());
        assert!(!state.has_valid_castling_rights());

        state.board.put_colored_pieces_at(ColoredPiece::WhiteRook, STARTING_QUEEN_SIDE_WR);
        state.board.clear_pieces_at(STARTING_BK);
        assert!(!state.has_valid_castling_rights());
        state.board.put_colored_pieces_at(ColoredPiece::BlackKing, Square::E4.to_mask());
        assert!(state.board.is_valid());
        assert!(!state.has_valid_castling_rights());
        let castling_info = state.context.castling_rights;
        state.context.castling_rights &= !0b00000011;
        assert!(state.has_valid_castling_rights());
        
        state.context.castling_rights = castling_info;
        state.board.clear_pieces_at(Square::E4.to_mask());
        state.board.put_colored_pieces_at(ColoredPiece::BlackKing, STARTING_BK);
        state.board.clear_pieces_at(STARTING_KING_SIDE_BR);
        assert!(state.board.is_valid());
        assert!(state.has_valid_castling_rights());

        state.board.put_colored_pieces_at(ColoredPiece::BlackRook, STARTING_KING_SIDE_BR);
        state.board.clear_pieces_at(STARTING_QUEEN_SIDE_BR);
        assert!(!state.has_valid_castling_rights());

        state.board.put_colored_pieces_at(ColoredPiece::BlackRook, STARTING_QUEEN_SIDE_BR);
        assert!(state.has_valid_castling_rights());
        
        state.context.castling_rights = 0b00000010;
        assert!(state.has_valid_castling_rights());
    }
    
    #[test]
    fn test_state_has_valid_double_pawn_push() {
        let state = State::blank();
        assert!(state.has_valid_double_pawn_push());
        
        let mut state = State::initial();
        assert_eq!(state.context.double_pawn_push, -1);
        assert!(state.has_valid_double_pawn_push());
        
        // todo
    }
    
    #[test]
    fn test_state_play_move() {
        // todo
    }
}