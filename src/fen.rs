use crate::enums::{Color, ColoredPiece, Square};
use crate::state::State;

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
    return true;
}

fn process_fen_castle(state: &mut State, fen_castle: &str) -> bool {
    if fen_castle == "-" {
        return true;
    }
    if fen_castle.len() > 4 {
        return false;
    }
    const indexer: &str = "KQkq";
    let mut already_seen = [false; 4];
    for c in fen_castle.chars() {
        let index = match indexer.find(c) {
            Some(i) => i,
            None => return false
        };
        if already_seen[index] {
            return false;
        }
        already_seen[index] = true;
        state.context.castling_info |= 1 << (3 - index);
    }
    return true;
}

fn process_en_passant_target_square(state: &mut State, fen_en_passant_target_square: &str) -> bool {
    if fen_en_passant_target_square == "-" { 
        return true; // no need to set state.context.double_pawn_push since it's already -1
    }

    let mut chars = fen_en_passant_target_square.chars();
    return match (chars.next(), chars.next(), chars.next()) {
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
            
            state.context.double_pawn_push = file_int as i8;
            
            true
        }
        _ => false,
    };
}

fn process_fen_halfmove_clock(state: &mut State, fen_halfmove_clock: &str) -> bool {
    let halfmove_clock_parsed = fen_halfmove_clock.parse::<u16>();
    return match halfmove_clock_parsed {
        Ok(halfmove_clock) => {
            if halfmove_clock > 100 {
                return false;
            }
            state.halfmove = halfmove_clock;
            true
        },
        Err(_) => false
    };
}

fn process_fen_fullmove(state: &mut State, fen_fullmove: &str) -> bool {
    let fullmove_parsed = fen_fullmove.parse::<u16>();
    return match fullmove_parsed {
        Ok(fullmove) => {
            state.halfmove = fullmove * 2 + state.side_to_move as u16;
            true
        },
        Err(_) => false
    };
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
                let dst =  unsafe { Square::from(row_from_top * 8 + file).to_mask() };
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
        
        return if state.is_valid() {
            Ok(state)
        } else {
            Err(FenParseError::InvalidState(fen.to_string()))
        }
    }

    // pub fn to_fen(&self) -> String {
    //     let mut fen_board = String::new();
    //     for row_from_top in 0..8 {
    //         let mut empty_count: u8 = 0;
    //         for file in 0..8 {
    //             let square_mask = 1 << (63 - (row_from_top * 8 + file));
    //             let piece_type = self.board.piece_type_at(square_mask);
    //             let piece_color
    //             if piece_type == PieceType::NoPieceType {
    //                 empty_count += 1;
    //             }
    //             else {
    //                 if empty_count > 0 {
    //                     fen_board.push_str(&empty_count.to_string());
    //                     empty_count = 0;
    //                 }
    //                 let colored_piece = unsafe { ColoredPiece::from(piece_type) };
    //                 fen_board.push(colored_piece.to_char());
    //             }
    //         }
    //         if empty_count > 0 {
    //             fen_board.push_str(&empty_count.to_string());
    //         }
    //         fen_board.push('/');
    //     }
    //     fen_board.pop();
    //     let turn = match self.turn {
    //         Color::White => 'w',
    //         Color::Black => 'b'
    //     };
    //     let mut castle = String::new();
    //     if self.wk_castle {
    //         castle.push('K');
    //     }
    //     if self.wq_castle {
    //         castle.push('Q');
    //     }
    //     if self.bk_castle {
    //         castle.push('k');
    //     }
    //     if self.bq_castle {
    //         castle.push('q');
    //     }
    //     if castle.is_empty() {
    //         castle.push('-');
    //     }
    //     let mut double_pawn_push = String::new();
    //     if self.double_pawn_push == -1 {
    //         double_pawn_push.push('-');
    //     }
    //     else {
    //         double_pawn_push.push((self.double_pawn_push as u8 + 'a' as u8) as char);
    //         double_pawn_push.push(if self.turn == Color::White {'6'} else {'3'});
    //     }
    //     format!("{} {} {} {} {} {}", fen_board, turn, castle, double_pawn_push, self.halfmove, self.halfmove_clock)
    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

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
        assert_eq!(state.context.castling_info, 0b00000000);
        
        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "KQkqq"), false);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "qq"), false);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "KQkq"), true);
        assert_eq!(state.context.castling_info, 0b00001111);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "Qkq"), true);
        assert_eq!(state.context.castling_info, 0b00000111);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, "qkK"), true);
        assert_eq!(state.context.castling_info, 0b00001011);

        let mut state = State::blank();
        assert_eq!(process_fen_castle(&mut state, " "), false);
    }

    #[test]
    fn test_process_fen_double_pawn_push() {
        let mut state = State::blank();
        assert!(process_en_passant_target_square(&mut state, "-"));
        assert_eq!(state.context.double_pawn_push, -1);
        
        let mut state = State::initial();

        assert!(process_en_passant_target_square(&mut state, "a6"));
        assert_eq!(state.context.double_pawn_push, 0);

        assert!(process_en_passant_target_square(&mut state, "f6"));
        assert_eq!(state.context.double_pawn_push, 5);
        
        assert!(!process_en_passant_target_square(&mut state, "f4"));
        assert!(!process_en_passant_target_square(&mut state, "f 3"));

        assert!(!process_en_passant_target_square(&mut state, "h3"));

        state.halfmove += 1;
        state.context.halfmove_clock += 1;
        state.side_to_move = Color::Black;
        
        assert!(process_en_passant_target_square(&mut state, "a3"));
        assert!(!process_en_passant_target_square(&mut state, " 3"));
        assert!(!process_en_passant_target_square(&mut state, "i3"));
        assert!(process_en_passant_target_square(&mut state, "a3"));
        assert_eq!(state.context.double_pawn_push, 0);

        assert!(!process_en_passant_target_square(&mut state, "d6"));
        assert!(process_en_passant_target_square(&mut state, "d3"));
        assert_eq!(state.context.double_pawn_push, 3);

        assert!(process_en_passant_target_square(&mut state, "h3"));
        assert_eq!(state.context.double_pawn_push, 7);
    }

    #[test]
    fn test_process_fn_halfmove_clock() {
        let mut state = State::initial();
        let is_valid = process_fen_halfmove_clock(&mut state, "0");
        assert!(is_valid);
        assert_eq!(state.halfmove, 0);
        let is_valid = process_fen_halfmove_clock(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.halfmove, 100);
        let is_valid = process_fen_halfmove_clock(&mut state, "101");
        assert!(!is_valid);
        let is_valid = process_fen_halfmove_clock(&mut state, "101a");
        assert!(!is_valid);
    }

    #[test]
    fn test_process_fen_fullmove() {
        let mut state = State::initial();
        
        let is_valid = process_fen_fullmove(&mut state, "0");
        assert!(is_valid);
        assert_eq!(state.halfmove, 0);
        state.side_to_move = Color::Black;
        let is_valid = process_fen_fullmove(&mut state, "1");
        assert!(is_valid);
        assert_eq!(state.halfmove, 3);
        
        let is_valid = process_fen_fullmove(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.halfmove, 201);
        state.side_to_move = Color::White;
        let is_valid = process_fen_fullmove(&mut state, "100");
        assert!(is_valid);
        assert_eq!(state.halfmove, 200);
        
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
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 2, "8");
        assert!(is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 3, "9");
        assert!(!is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 3, "12R4");
        assert!(is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 4, "1Qrrrrrr");
        assert!(is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 5, "bnbNbNb");
        assert!(!is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 8, "8");
        assert!(!is_valid);
        assert!(state.board.is_valid());
        let is_valid = process_fen_board_row(&mut state, 7, "7 ");
        assert!(!is_valid);
        assert!(state.board.is_valid());
        
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
        assert!(state.board.is_valid());
        state.context.castling_info = 0b00001111;
        state.position_count = HashMap::from([(state.board.zobrist_hash(), 1)]);
        assert_eq!(state, State::initial());
    }
    
    #[test]
    fn test_process_fen_board() { // todo: finish
        let mut state = State::blank();

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_valid());
        assert_eq!(state, State::initial());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());
        assert!(!state.board.is_valid());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR ";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_err());
        assert!(!state.board.is_valid());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_valid());
        assert_eq!(state, State::initial());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_valid());
        assert_eq!(state, State::initial());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
        let result = process_fen_board(&mut state, fen_board);
        assert!(result.is_ok());
        assert!(state.board.is_valid());
        assert_eq!(state, State::initial());

        let fen_board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    }
}