use crate::utils::{Color, ColoredPiece, PieceType, Square};
use crate::state::State;

pub const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Eq, PartialEq, Debug)]
pub enum FenParseError {
    InvalidFieldCount(usize),
    InvalidRankCount(usize),
    InvalidRow(String),
    InvalidSideToMove(String),
    InvalidCastle(String),
    InvalidEnPassantTarget(String),
    InvalidHalfmoveClock(String),
    InvalidFullmoveCounter(String),
    InvalidState(String)
}

fn process_fen_side_to_move(state: &mut State, fen_side_to_move: &str) -> bool {
    if fen_side_to_move == "w" {
        state.side_to_move = Color::White;
    }
    else if fen_side_to_move == "b" {
        state.side_to_move = Color::Black;
    }
    else {
        return false;
    }
    true
}

fn process_fen_castle(state: &mut State, fen_castle: &str) -> bool {
    if fen_castle == "-" {
        return true;
    }
    if fen_castle.len() > 4 {
        return false;
    }
    const INDEXER: &str = "KQkq";
    let mut already_seen = [false; 4];
    for c in fen_castle.chars() {
        let index = match INDEXER.find(c) {
            Some(i) => i,
            None => return false
        };
        if already_seen[index] {
            return false;
        }
        already_seen[index] = true;
        state.context.borrow_mut().castling_rights |= 1 << (3 - index);
    }
    true
}

fn process_en_passant_target_square(state: &mut State, fen_en_passant_target_square: &str) -> bool {
    if fen_en_passant_target_square == "-" { 
        return true; // no need to set state.context.double_pawn_push since it's already -1
    }

    let mut chars = fen_en_passant_target_square.chars();
    match (chars.next(), chars.next(), chars.next()) {
        (Some(file), Some(rank), None) => {
            if !file.is_ascii_alphabetic() {
                return false;
            }

            let file = file.to_ascii_lowercase();
            let file_int = file as u8 - 'a' as u8;
            if file_int > 7 {
                return false;
            }
            
            if !rank.is_ascii_digit() {
                return false;
            }
            
            let rank = rank.to_digit(10).unwrap();
            if match state.side_to_move { // expect side_to_move to be set first
                Color::White => rank != 6,
                Color::Black => rank != 3
            } {
                return false;
            }
            
            state.context.borrow_mut().double_pawn_push = file_int as i8;
            
            true
        }
        _ => false,
    }
}

fn process_fen_halfmove_clock(state: &mut State, fen_halfmove_clock: &str) -> bool {
    let halfmove_clock_parsed = fen_halfmove_clock.parse::<u8>();
    match halfmove_clock_parsed {
        Ok(halfmove_clock) => {
            if halfmove_clock > 100 {
                return false;
            }
            state.context.borrow_mut().halfmove_clock = halfmove_clock;
            true
        },
        Err(_) => false
    }
}

fn process_fen_fullmove(state: &mut State, fen_fullmove: &str) -> bool {
    let fullmove_parsed = fen_fullmove.parse::<u16>();
    match fullmove_parsed {
        Ok(fullmove) => {
            if fullmove < 1 {
                return false;
            }
            state.halfmove = (fullmove - 1) * 2 + state.side_to_move as u16;
            true
        },
        Err(_) => false
    }
}

fn process_fen_board_row(state: &mut State, row_from_top: u8, row: &str) -> bool {
    if row_from_top > 7 {
        return false;
    }
    if row.len() > 8 || row.is_empty() {
        return false;
    }
    let mut file = 0;
    for c in row.chars() {
        match c {
            _ if c.is_ascii_digit() => {
                file += c.to_digit(10).unwrap() as u8;
                if file > 8 {
                    return false;
                }
                continue;
            },
            _ if c.is_ascii_alphabetic() => {
                let colored_piece = ColoredPiece::from_char(c);
                if colored_piece == ColoredPiece::NoPiece {
                    return false;
                }
                let dst =  unsafe { Square::from(row_from_top * 8 + file) };
                state.board.put_colored_piece_at(colored_piece, dst);
            },
            _ => {
                return false;
            }
        }
        file += 1;
    }
    file == 8
}

fn process_fen_board(state: &mut State, fen_board: &str) -> Result<State, FenParseError> {
    let mut row_from_top = 0;
    let rows = fen_board.split('/');
    let row_count = rows.clone().count();
    if row_count != 8 {
        return Err(FenParseError::InvalidRankCount(row_count));
    }
    for row in rows {
        let is_valid_row = process_fen_board_row(state, row_from_top, row);
        if !is_valid_row {
            return Err(FenParseError::InvalidRow(row.to_string()));
        }
        row_from_top += 1;
    }
    Ok(State::blank())
}

impl State {
    pub fn from_fen(fen: &str) -> Result<State, FenParseError> {
        let mut state = State::blank();
        
        let fen_parts: Vec<&str> = fen.split_ascii_whitespace().collect();
        if fen_parts.len() != 6 {
            return Err(FenParseError::InvalidFieldCount(fen_parts.len()));
        }
        
        let [
            fen_board, 
            fen_side_to_move, 
            fen_castle, 
            fen_double_pawn_push, 
            fen_halfmove_clock, 
            fen_fullmove
        ] = match &fen_parts[..] {
            [
                board, 
                side_to_move, 
                castle, 
                double_pawn_push, 
                halfmove_clock, 
                fullmove
            ] => [board, side_to_move, castle, double_pawn_push, halfmove_clock, fullmove],
            _ => return Err(FenParseError::InvalidFieldCount(fen_parts.len())),
        };
        
        let is_fen_side_to_move_valid = process_fen_side_to_move(&mut state, fen_side_to_move);
        if !is_fen_side_to_move_valid {
            return Err(FenParseError::InvalidSideToMove(fen_side_to_move.to_string()));
        }
        
        let is_fen_castle_valid = process_fen_castle(&mut state, fen_castle);
        if !is_fen_castle_valid {
            return Err(FenParseError::InvalidCastle(fen_castle.to_string()));
        }
        
        let is_fen_double_pawn_push_valid = process_en_passant_target_square(&mut state, fen_double_pawn_push);
        if !is_fen_double_pawn_push_valid {
            return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
        }
        
        let is_fen_halfmove_clock_valid = process_fen_halfmove_clock(&mut state, fen_halfmove_clock);
        if !is_fen_halfmove_clock_valid {
            return Err(FenParseError::InvalidHalfmoveClock(fen_halfmove_clock.to_string()));
        }
        
        let is_fen_fullmove_valid = process_fen_fullmove(&mut state, fen_fullmove);
        if !is_fen_fullmove_valid {
            return Err(FenParseError::InvalidFullmoveCounter(fen_fullmove.to_string()));
        }
        
        let fen_board_result = process_fen_board(&mut state, fen_board);
        if fen_board_result.is_err() {
            return fen_board_result;
        }

        let zobrist_hash = state.board.calc_zobrist_hash();
        state.board.zobrist_hash = zobrist_hash;
        state.context.borrow_mut().zobrist_hash = zobrist_hash;
        
        if state.is_unequivocally_valid() {
            Ok(state)
        } else {
            Err(FenParseError::InvalidState(fen.to_string()))
        }
    }

    fn get_fen_board(&self) -> String {
        let mut fen_board = String::new();
        for row_from_top in 0..8 {
            let mut empty_count: u8 = 0;
            for file in 0..8 {
                let square = unsafe { Square::from(row_from_top * 8 + file) };
                let piece_type = self.board.get_piece_type_at(square);
                if piece_type == PieceType::NoPieceType {
                    empty_count += 1;
                }
                else {
                    if empty_count > 0 {
                        fen_board.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let is_black = self.board.color_masks[Color::Black as usize] & square.get_mask() != 0;
                    let colored_piece = ColoredPiece::from(Color::from(is_black), piece_type);
                    fen_board.push(colored_piece.to_char());
                }
            }
            if empty_count > 0 {
                fen_board.push_str(&empty_count.to_string());
            }
            fen_board.push('/');
        }
        fen_board.pop();
        fen_board
    }

    fn get_fen_side_to_move(&self) -> char {
        match self.side_to_move {
            Color::White => 'w',
            Color::Black => 'b'
        }
    }

    fn get_fen_castling_info(&self) -> String {
        let context = self.context.borrow(); 
        if context.castling_rights == 0 {
            return "-".to_string();
        }
        let mut castling_info = String::with_capacity(4);
        let castling_chars = ['K', 'Q', 'k', 'q'];
        let mask = 0b1000;
        for i in 0..4 {
            if context.castling_rights & mask >> i != 0 {
                castling_info.push(castling_chars[i]);
            }
        }
        castling_info
    }

    fn get_fen_en_passant_target(&self) -> String {
        let context = self.context.borrow();
        if context.double_pawn_push == -1 {
            return "-".to_string();
        }
        let file = (context.double_pawn_push + 'a' as i8) as u8;
        let rank = match self.side_to_move {
            Color::White => 6,
            Color::Black => 3
        };
        format!("{}{}", file as char, rank)
    }

    fn get_fen_halfmove_clock(&self) -> String {
        self.context.borrow().halfmove_clock.to_string()
    }

    fn get_fen_fullmove(&self) -> String {
        ((self.halfmove - self.side_to_move as u16) / 2 + 1).to_string()
    }

    pub fn to_fen(&self) -> String {
        let fen_board = self.get_fen_board();
        let side_to_move = self.get_fen_side_to_move();
        let castling_info = self.get_fen_castling_info();
        let en_passant_target = self.get_fen_en_passant_target();
        let halfmove_clock = self.get_fen_halfmove_clock();
        let fullmove = self.get_fen_fullmove();
        [fen_board, side_to_move.to_string(), castling_info, en_passant_target, halfmove_clock, fullmove].join(" ")
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_squares_from_mask_iter;
    use super::*;
    use crate::utils::masks::{RANK_2, RANK_3, RANK_6, RANK_7};
    use crate::state::board::Board;
    use crate::state::State;

    #[test]
    fn test_process_fen_side_to_move() {
        let mut state = State::blank();
        assert_eq!(process_fen_side_to_move(&mut state, "w"), true);
        assert_eq!(state.side_to_move, Color::White);
        
        let mut state = State::blank();
        assert_eq!(process_fen_side_to_move(&mut state, "b"), true);
        assert_eq!(state.side_to_move, Color::Black);
        
        let mut state = State::blank();
        assert_eq!(process_fen_side_to_move(&mut state, ""), false);
    }

    #[test]
    fn test_process_fen_castle() {
        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "-"), true);
        assert_eq!(state.context.borrow().castling_rights, 0b00000000);
        
        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "KQkqq"), false);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "qq"), false);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "KQkq"), true);
        assert_eq!(state.context.borrow().castling_rights, 0b00001111);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "Qkq"), true);
        assert_eq!(state.context.borrow().castling_rights, 0b00000111);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "qkK"), true);
        assert_eq!(state.context.borrow().castling_rights, 0b00001011);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, " "), false);
    }

    #[test]
    fn test_process_fen_double_pawn_push() {
        let mut state = State::blank();
        assert!(process_en_passant_target_square(&mut state, "-"));
        assert_eq!(state.context.borrow().double_pawn_push, -1);
        
        let mut state = State::initial();

        assert!(process_en_passant_target_square(&mut state, "a6"));
        assert_eq!(state.context.borrow().double_pawn_push, 0);

        assert!(process_en_passant_target_square(&mut state, "f6"));
        assert_eq!(state.context.borrow().double_pawn_push, 5);
        
        assert!(!process_en_passant_target_square(&mut state, "f4"));
        assert!(!process_en_passant_target_square(&mut state, "f 3"));

        assert!(!process_en_passant_target_square(&mut state, "h3"));

        state.halfmove += 1;
        state.context.borrow_mut().halfmove_clock += 1;
        state.side_to_move = Color::Black;
        
        assert!(process_en_passant_target_square(&mut state, "a3"));
        assert!(!process_en_passant_target_square(&mut state, " 3"));
        assert!(!process_en_passant_target_square(&mut state, "i3"));
        assert!(process_en_passant_target_square(&mut state, "a3"));
        assert_eq!(state.context.borrow().double_pawn_push, 0);

        assert!(!process_en_passant_target_square(&mut state, "d6"));
        assert!(process_en_passant_target_square(&mut state, "d3"));
        assert_eq!(state.context.borrow().double_pawn_push, 3);

        assert!(process_en_passant_target_square(&mut state, "h3"));
        assert_eq!(state.context.borrow().double_pawn_push, 7);
    }

    #[test]
    fn test_process_fen_halfmove_clock() {
        let mut state = State::initial();
        let is_valid = process_fen_halfmove_clock(&mut state, "0");
        assert!(is_valid);
        assert_eq!(state.context.borrow().halfmove_clock, 0);
        let is_valid = process_fen_halfmove_clock(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.context.borrow().halfmove_clock, 100);
        let is_valid = process_fen_halfmove_clock(&mut state, "101");
        assert!(!is_valid);
        let is_valid = process_fen_halfmove_clock(&mut state, "101a");
        assert!(!is_valid);
    }

    #[test]
    fn test_process_fen_fullmove() {
        let mut state = State::initial();
        
        let is_valid = process_fen_fullmove(&mut state, "0");
        assert!(!is_valid);

        let is_valid = process_fen_fullmove(&mut state, "1");
        assert!(is_valid);
        assert_eq!(state.halfmove, 0);

        state.side_to_move = Color::Black;
        let is_valid = process_fen_fullmove(&mut state, "1");
        assert!(is_valid);
        assert_eq!(state.halfmove, 1);
        
        let is_valid = process_fen_fullmove(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.halfmove, 199);

        state.side_to_move = Color::White;
        let is_valid = process_fen_fullmove(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.halfmove, 198);
        
        let is_valid = process_fen_fullmove(&mut state, "101a");
        assert!(!is_valid);
    }

    #[test]
    fn test_process_fen_board_row() {
        let mut state = State::blank();
        
        let is_valid = process_fen_board_row(&mut state, 0, "rnbqkbnr");
        assert!(is_valid);
        let is_valid = process_fen_board_row(&mut state, 1, "4K3");
        assert!(is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 2, "8");
        assert!(is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 3, "9");
        assert!(!is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 3, "12R4");
        assert!(is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 4, "1Qrrrrrr");
        assert!(is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 5, "bnbNbNb");
        assert!(!is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 8, "8");
        assert!(!is_valid);
        assert!(state.board.is_unequivocally_valid());
        let is_valid = process_fen_board_row(&mut state, 7, "7 ");
        assert!(!is_valid);
        assert!(state.board.is_unequivocally_valid());
        
        let mut state = State::blank();
        
        assert_eq!(state, State::blank());
        let is_valid = process_fen_board_row(&mut state, 0, "rnbqkbnr");
        assert!(is_valid);
        let is_valid = process_fen_board_row(&mut state, 1, "pppppppp");
        assert!(is_valid);
        let is_valid = process_fen_board_row(&mut state, 6, "PPPPPPPP");
        assert!(is_valid);
        let is_valid = process_fen_board_row(&mut state, 7, "RNBQKBNR");
        assert!(is_valid);
        assert!(state.board.is_unequivocally_valid());
        state.context.borrow_mut().castling_rights = 0b00001111;
        state.context.borrow_mut().zobrist_hash = state.board.zobrist_hash;
        assert_eq!(state, State::initial());
    }
    
    #[test]
    fn test_process_fen_board() {
        let mut state = State::blank();
        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_unequivocally_valid());
        state.context.borrow_mut().castling_rights = 0b00001111;
        state.context.borrow_mut().zobrist_hash = state.board.zobrist_hash;
        assert_eq!(state, State::initial());
        
        let mut state = State::blank();
        let fen_board = "8/8/8/8/8/8/k7/7K";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_unequivocally_valid());
        let mut expected_board = Board::blank();
        expected_board.put_colored_piece_at(ColoredPiece::BlackKing, Square::A2);
        expected_board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::H1);
        assert_eq!(state.board, expected_board);

        let mut state = State::blank();
        let fen_board = "8/8/8/8/8/8/k7/7K/8";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());

        let mut state = State::blank();
        let fen_board = "8/8/8/8/8/k7/7K";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());

        let mut state = State::blank();
        let fen_board = "8/8/8//8/8/8/k7/7K";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());

        let mut state = State::blank();
        let fen_board = "8/8/8//8/8/k7/7K";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_fen() {
        let fen = INITIAL_FEN;
        let state = State::from_fen(fen);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(state.board.is_unequivocally_valid());
        assert_eq!(state, State::initial());
        
        let fen = "8/8/8/8/8/8/k7/7K b - - 99 88";
        let state = State::from_fen(fen);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(state.is_unequivocally_valid());
        let mut expected_state = State::blank();
        expected_state.board.put_colored_piece_at(ColoredPiece::BlackKing, Square::A2);
        expected_state.board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::H1);
        expected_state.halfmove = 175;
        expected_state.side_to_move = Color::Black;
        expected_state.context.borrow_mut().halfmove_clock = 99;
        expected_state.context.borrow_mut().zobrist_hash = expected_state.board.zobrist_hash;
        assert_eq!(state, expected_state);
        
        let fen = "r2qk2r/8/8/7p/8/8/8/R2QK2R w KQkq h6 0 6";
        let state = State::from_fen(fen);
        assert!(state.is_ok());
        let state = state.unwrap();
        assert!(state.board.is_unequivocally_valid());
        let mut expected_state = State::initial();
        let clear_mask = Square::B8.get_mask() | Square::B1.get_mask() |
            Square::C8.get_mask() | Square::C1.get_mask() |
            Square::F8.get_mask() | Square::F1.get_mask() |
            Square::G8.get_mask() | Square::G1.get_mask() |
            RANK_7 | RANK_6 |
            RANK_3 | RANK_2;
        for square in get_squares_from_mask_iter(clear_mask) {
            let colored_piece = expected_state.board.get_colored_piece_at(square);
            if colored_piece != ColoredPiece::NoPiece {
                expected_state.board.remove_colored_piece_at(colored_piece, square);
            }
        }
        expected_state.board.put_colored_piece_at(ColoredPiece::BlackPawn, Square::H5);
        expected_state.halfmove = 10;
        expected_state.context.borrow_mut().double_pawn_push = 7;
        expected_state.context.borrow_mut().zobrist_hash = expected_state.board.zobrist_hash;
        assert_eq!(state, expected_state);
    }
    
    #[test]
    fn test_to_fen() {
        let mut state = State::initial();
        
        let fen = state.to_fen();
        let expected_fen = INITIAL_FEN;
        assert_eq!(fen, expected_fen);
        
        state.halfmove += 1;
        state.context.borrow_mut().halfmove_clock += 1;
        state.side_to_move = Color::Black;
        state.board.put_colored_piece_at(ColoredPiece::BlackQueen, Square::D4);
        state.board.remove_colored_piece_at(ColoredPiece::WhiteRook, Square::H1);
        state.context.borrow_mut().castling_rights &= !0b1000;
        let fen = state.to_fen();
        let expected_fen = "rnbqkbnr/pppppppp/8/8/3q4/8/PPPPPPPP/RNBQKBN1 b Qkq - 1 1";
    }
    
    #[test]
    fn test_fen() {
        let fen = "8/1P1n1B2/5P2/4pkNp/1PQ4K/p2p2P1/8/3R1N2 w - - 0 1";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        assert_eq!(state_result.unwrap().to_fen(), fen);

        let fen = "1k2N1K1/4Q3/6p1/2B2B2/p1PPb3/2P2Nb1/2r5/n7 b - - 36 18";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_err());
        assert_eq!(state_result.err().unwrap(), FenParseError::InvalidState(fen.to_string()));

        let fen = "1k2N1K1/4Q3/6p1/2B2B2/p1PPb3/2P2Nb1/2r5/n7 b - - 35 18";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok(), "{:?}", state_result);
        assert_eq!(state_result.unwrap().to_fen(), fen);
        
        let fen = "r3k3/P3P3/1B3q2/N3P2P/R6N/8/np2b2p/1K3n2 w q - 100 96";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        assert_eq!(state_result.unwrap().to_fen(), fen);

        let fen = "nb4K1/2N4p/8/3P1rk1/1r2P3/5p2/3P1Q2/B2R1b2 b - - 0 1";
        let state_result = State::from_fen(fen);
        assert!(state_result.is_ok());
        assert_eq!(state_result.unwrap().to_fen(), fen);
    }
}