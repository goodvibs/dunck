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
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq
    
    // updated after every move
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
        
        let state = State::initial();
        assert_eq!(state.context.double_pawn_push, -1);
        assert!(state.has_valid_double_pawn_push());
        
        // todo
    }
    
    #[test]
    fn test_state_play_move() {
        // todo
    }
}