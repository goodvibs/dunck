use std::collections::HashMap;
use crate::board::Board;
use crate::r#move::*;
use crate::charboard::*;
use crate::attacks::*;
use crate::bitboard::unpack_bb;
use crate::enums::*;
use crate::masks::*;
use crate::pgn::pgn_move_tree::PgnParseError;
use crate::pgn::PgnMoveTree;
use crate::preload::ZOBRIST_TABLE;

#[derive(Eq, PartialEq, Clone)]
pub enum Termination {
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule
}

impl Termination {
    pub fn is_conclusive(&self) -> bool {
        self == &Termination::Checkmate
    }

    pub fn is_draw(&self) -> bool {
        !self.is_conclusive()
    }
}

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

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct StateContext {
    // copied from previous
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_info: u8, // 0, 0, 0, 0, wk, wq, bk, bq
    
    // recomputed on every move
    pub captured_piece: PieceType,
    pub previous: Option<Box<StateContext>>
}

impl StateContext {
    pub fn new(halfmove_clock: u8, double_pawn_push: i8, castling_info: u8, captured_piece: PieceType, previous: Option<Box<StateContext>>) -> StateContext {
        StateContext {
            halfmove_clock,
            double_pawn_push,
            castling_info,
            captured_piece,
            previous
        }
    }
    
    pub fn initial() -> StateContext {
        StateContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_info: 0b00001111,
            captured_piece: PieceType::NoPieceType,
            previous: None
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct State {
    pub board: Board,
    pub in_check: bool,
    pub position_count: HashMap<u64, u8>,
    pub turn: Color,
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
            position_count,
            turn: Color::White,
            halfmove: 0,
            termination: None,
            context: Box::new(StateContext::initial())
        }
    }

    pub fn initial() -> State {
        let board = Board::initial();
        let position_count: HashMap<u64, u8> = HashMap::from([(board.zobrist_hash(), 1)]);
        State {
            board,
            in_check: false,
            position_count,
            turn: Color::White,
            halfmove: 0,
            termination: None,
            context: Box::new(StateContext::initial())
        }
    }

    pub fn from_fen(fen: &str) -> Result<State, FenParseError> {
        let mut state = State::blank();
        let mut fen_iter = fen.split_ascii_whitespace();
        let field_count = fen_iter.clone().count();
        if field_count != 6 {
            return Err(FenParseError::InvalidFieldCount(field_count));
        }
        let fen_board = fen_iter.next().unwrap();
        let fen_turn = fen_iter.next().unwrap();
        if fen_turn == "w" {
            state.turn = Color::White;
        }
        else if fen_turn == "b" {
            state.turn = Color::Black;
        }
        else {
            return Err(FenParseError::InvalidSideToMove(fen_turn.to_string()));
        }
        let fen_castle = fen_iter.next().unwrap();
        if fen_castle != "-" {
            if fen_castle.len() > 4 {
                return Err(FenParseError::InvalidCastle(fen_castle.to_string()));
            }
            for c in fen_castle.chars() {
                match c {
                    'K' => state.context.castling_info |= 0b00001000,
                    'Q' => state.context.castling_info |= 0b00000100,
                    'k' => state.context.castling_info |= 0b00000010,
                    'q' => state.context.castling_info |= 0b00000001,
                    _ => return Err(FenParseError::InvalidCastle(fen_castle.to_string()))
                }
            }
        }
        let fen_double_pawn_push = fen_iter.next().unwrap();
        if fen_double_pawn_push != "-" {
            if fen_double_pawn_push.len() > 2 {
                return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
            }
            let file = fen_double_pawn_push.chars().next().unwrap();
            if !file.is_ascii_alphabetic() {
                return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
            }
            let file = file.to_ascii_lowercase();
            let file = file as u8 - 'a' as u8;
            if file > 7 {
                return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
            }
            let rank = fen_double_pawn_push.chars().last().unwrap();
            if !rank.is_ascii_digit() {
                return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
            }
            let rank = rank.to_digit(10).unwrap();
            if rank != 3 && rank != 6 {
                return Err(FenParseError::InvalidEnPassantTarget(fen_double_pawn_push.to_string()));
            }
            state.context.double_pawn_push = file as i8;
        }
        let fen_halfmove_clock = fen_iter.next().unwrap();
        if fen_halfmove_clock != "-" {
            let halfmove_clock_parsed = fen_halfmove_clock.parse::<u16>();
            if halfmove_clock_parsed.is_err() {
                return Err(FenParseError::InvalidHalfmoveClock(fen_halfmove_clock.to_string()));
            }
            state.halfmove = halfmove_clock_parsed.unwrap();
        }
        let fen_fullmove = fen_iter.next().unwrap();
        if fen_fullmove != "-" {
            let fullmove_parsed = fen_fullmove.parse::<u16>();
            if fullmove_parsed.is_err() {
                return Err(FenParseError::InvalidFullmoveCounter(fen_fullmove.to_string()));
            }
            state.halfmove = fullmove_parsed.unwrap() + (state.turn == Color::Black) as u16;
        }
        let mut row_from_top = 0;
        let rows = fen_board.split('/');
        let row_count = rows.clone().count();
        if row_count != 8 {
            return Err(FenParseError::InvalidRankCount(row_count));
        }
        for row in rows {
            if row.len() > 8 || row.is_empty() {
                return Err(FenParseError::InvalidRow(row.to_string()));
            }
            let mut file = 0;
            for c in row.chars() {
                let dst =  1 << (63 - (row_from_top * 8 + file));
                match c {
                    c if c.is_ascii_whitespace() => {
                        continue;
                    },
                    _ if c.is_ascii_digit() => {
                        file += c.to_digit(10).unwrap() as usize - 1;
                        if file > 8 {
                            return Err(FenParseError::InvalidRow(row.to_string()));
                        }
                    },
                    _ if c.is_ascii_alphabetic() => {
                        let colored_piece = ColoredPiece::from_char(c);
                        if colored_piece == ColoredPiece::NoPiece {
                            return Err(FenParseError::InvalidRow(row.to_string()));
                        }
                        let piece_type = colored_piece.get_piece_type();
                        let color = colored_piece.get_color();
                        state.board.bb_by_piece_type[piece_type as usize] |= dst;
                        state.board.bb_by_color[color as usize] |= dst;
                    },
                    _ => {
                        return Err(FenParseError::InvalidRow(row.to_string()));
                    }
                }
                file += 1;
            }
            row_from_top += 1;
        }
        return if state.is_valid() {
            Ok(state)
        } else {
            Err(FenParseError::InvalidState(fen.to_string()))
        }
    }

    pub fn from_pgn(pgn: &str) -> Result<State, PgnParseError> {
        let move_tree = PgnMoveTree::from_pgn(pgn);
        todo!()
    }

    pub fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }
    
    pub fn get_pseudolegal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        
        let colored_piece_adjustment = self.turn as usize * ColoredPiece::COLOR_DIFFERENCE as usize;
        let all_occupancy_bb = self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize];
        
        let pawns_bb = self.board.bb_by_piece_type[PieceType::Pawn as usize + colored_piece_adjustment];
        let pawn_srcs = unpack_bb(pawns_bb);
        let promotion_rank = match self.turn {
            Color::White => RANK_8,
            Color::Black => RANK_1
        };
        
        // pawn captures excluding en passant
        for src in pawn_srcs.clone() {
            let captures = pawn_attacks(src, self.turn) & self.board.bb_by_color[self.turn.flip() as usize];
            for dst in unpack_bb(captures) {
                let move_src = unsafe { Square::from(src.leading_zeros() as u8) };
                let move_dst = unsafe { Square::from(dst.leading_zeros() as u8) };
                if dst & promotion_rank != 0 {
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PromoteToQueen));
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PromoteToKnight));
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PromoteToRook));
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PromoteToBishop));
                }
                else {
                    moves.push(Move::new(move_src, move_dst, MoveFlag::PawnMove));
                }
            }
        }
        
        // en passant
        let pawn_double_push_rank = match self.turn {
            Color::White => RANK_5,
            Color::Black => RANK_4
        };
        let (src_offset, dst_offset) = match self.turn {
            Color::White => (24, 16),
            Color::Black => (32, 40)
        };
        if (*self.context).double_pawn_push != -1 { // if en passant is possible
            for direction in [-1, 1].iter() { // left and right
                let double_pawn_push_file = self.context.double_pawn_push as i32 + direction;
                if double_pawn_push_file >= 0 && double_pawn_push_file <= 7 { // if within bounds
                    let double_pawn_push_file_mask = FILE_A >> double_pawn_push_file;
                    if pawns_bb & double_pawn_push_file_mask & RANK_5 != 0 {
                        let move_src = unsafe { Square::from((src_offset + double_pawn_push_file) as u8) };
                        let move_dst = unsafe { Square::from((dst_offset + self.context.double_pawn_push) as u8) };
                        moves.push(Move::new(move_src, move_dst, MoveFlag::EnPassant));
                    }
                }
            }
        }
        
        // pawn pushes
        let single_push_rank = match self.turn {
            Color::White => RANK_3,
            Color::Black => RANK_6
        };
        for src_bb in pawn_srcs.iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            
            // single moves
            let single_move_dst = pawn_moves(*src_bb, self.turn) & !all_occupancy_bb;
            let single_move_dst_square = unsafe { Square::from(single_move_dst.leading_zeros() as u8) };
            
            if single_move_dst == 0 { // if no single moves
                continue;
            }
            
            // double push
            if single_move_dst & single_push_rank != 0 {
                let double_move_dst = pawn_moves(single_move_dst, Color::White) & !all_occupancy_bb;
                if double_move_dst != 0 {
                    unsafe {
                        let double_move_dst_square = Square::from(double_move_dst.leading_zeros() as u8);
                        moves.push(Move::new(src_square, double_move_dst_square, MoveFlag::PawnDoubleMove));
                    }
                }
            }
            else if single_move_dst & promotion_rank != 0 { // promotion
                moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PromoteToQueen));
                moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PromoteToKnight));
                moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PromoteToRook));
                moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PromoteToBishop));
                continue;
            }

            // single push
            moves.push(Move::new(src_square, single_move_dst_square, MoveFlag::PawnMove));
        }
        
        // knight moves
        let knights_bb = self.board.bb_by_piece_type[PieceType::Knight as usize + colored_piece_adjustment];
        for src_bb in unpack_bb(knights_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let knight_moves = knight_attacks(*src_bb) & !all_occupancy_bb;
            for dst_bb in unpack_bb(knight_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new(src_square, dst_square, MoveFlag::KnightMove));
            }
        }
        
        // bishop moves
        let bishops_bb = self.board.bb_by_piece_type[PieceType::Bishop as usize + colored_piece_adjustment];
        for src_bb in unpack_bb(bishops_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let bishop_moves = bishop_attacks(*src_bb, all_occupancy_bb) & !all_occupancy_bb;
            for dst_bb in unpack_bb(bishop_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new(src_square, dst_square, MoveFlag::BishopMove));
            }
        }
        
        // rook moves
        let rooks_bb = self.board.bb_by_piece_type[PieceType::Rook as usize + colored_piece_adjustment];
        for src_bb in unpack_bb(rooks_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let rook_moves = rook_attacks(*src_bb, all_occupancy_bb) & !all_occupancy_bb;
            for dst_bb in unpack_bb(rook_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new(src_square, dst_square, MoveFlag::RookMove));
            }
        }
        
        // queen moves
        let queens_bb = self.board.bb_by_piece_type[PieceType::Queen as usize + colored_piece_adjustment];
        for src_bb in unpack_bb(queens_bb).iter() {
            let src_square = unsafe { Square::from(src_bb.leading_zeros() as u8) };
            let queen_moves = (rook_attacks(*src_bb, all_occupancy_bb) | bishop_attacks(*src_bb, all_occupancy_bb)) & !all_occupancy_bb;
            for dst_bb in unpack_bb(queen_moves).iter() {
                let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
                moves.push(Move::new(src_square, dst_square, MoveFlag::QueenMove));
            }
        }
        
        // king moves
        let king_src_bb = self.board.bb_by_piece_type[PieceType::King as usize + colored_piece_adjustment];
        let king_src_square = unsafe { Square::from(king_src_bb.leading_zeros() as u8) };
        let king_moves = king_attacks(king_src_bb) & !all_occupancy_bb;
        for dst_bb in unpack_bb(king_moves).iter() {
            let dst_square = unsafe { Square::from(dst_bb.leading_zeros() as u8) };
            moves.push(Move::new(king_src_square, dst_square, MoveFlag::KingMove));
        }
        
        moves
    }
    
    pub fn get_moves(&self) -> Vec<Move> {
        // todo
        self.get_pseudolegal_moves()
    }
    
    pub fn play_move(&mut self, mv: Move) {
        let (src_square, dst_square, flag) = mv.unpack();
        let src = 1 << (63 - src_square as u8);
        let dst = 1 << (63 - dst_square as u8);
        let src_dst = src | dst;
        let mut new_context = StateContext::new(self.context.halfmove_clock + 1, -1, self.context.castling_info.clone(), PieceType::NoPieceType, Some(self.context.clone()));
        let color_adjustment = self.turn as usize * ColoredPiece::COLOR_DIFFERENCE as usize;
        let castling_color_adjustment = self.turn as usize * 2;
        if flag != MoveFlag::EnPassant && self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize] & dst != 0 {
            let captured_piece = self.board.piece_type_at(dst);
            new_context.captured_piece = captured_piece;
            new_context.halfmove_clock = 0;
            self.board.bb_by_color[self.turn.flip() as usize] &= !dst;
        }
        let previous_castling_rights = self.context.castling_info.clone();
        self.board.bb_by_color[self.turn as usize] ^= src_dst; // works for all moves except the rook in castling
        self.board.bb_by_color[self.turn.flip() as usize] &= !dst; // clear opponent's piece if any
        match flag {
            MoveFlag::PawnMove => { // can be a single pawn push or capture (non-promotion)
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Pawn as usize] |= dst; // since dst could contain a piece, xor is not safe
                new_context.halfmove_clock = 0;
            }
            MoveFlag::PawnDoubleMove => { // pawn double push, not a capture or promotion
                self.board.bb_by_piece_type[PieceType::Pawn as usize] ^= src_dst; // since dst is empty, xor is safe
                new_context.double_pawn_push = (src_square as u8 % 8) as i8;
                new_context.halfmove_clock = 0;
            }
            MoveFlag::EnPassant => { // en passant capture
                let en_passant_capture = ((dst << 8) * self.turn as u64) | ((dst >> 8) * self.turn.flip() as u64);
                self.board.bb_by_piece_type[PieceType::Pawn as usize] ^= src_dst | en_passant_capture;
                new_context.captured_piece = PieceType::Pawn;
                new_context.halfmove_clock = 0;
            }
            MoveFlag::KnightMove => {
                self.board.bb_by_piece_type[PieceType::Knight as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Knight as usize] |= dst;
            }
            MoveFlag::BishopMove => {
                self.board.bb_by_piece_type[PieceType::Bishop as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Bishop as usize] |= dst;
            }
            MoveFlag::RookMove => {
                self.board.bb_by_piece_type[PieceType::Rook as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Rook as usize] |= dst;
                let is_king_side = src & (1u64 << (self.turn as u64 * 7 * 8));
                let is_queen_side = src & (0b10000000u64 << (self.turn as u64 * 7 * 8));
                let king_side_mask = (is_king_side != 0) as u8 * (0b00001000 >> castling_color_adjustment);
                let queen_side_mask = (is_queen_side != 0) as u8 * (0b00000100 >> castling_color_adjustment);
                new_context.castling_info &= !(king_side_mask | queen_side_mask);
            }
            MoveFlag::QueenMove => {
                self.board.bb_by_piece_type[PieceType::Queen as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Queen as usize] |= dst;
            }
            MoveFlag::KingMove => {
                self.board.bb_by_piece_type[PieceType::King as usize] ^= src_dst; // since dst is empty, xor is safe
                new_context.castling_info &= !0b00001100 >> castling_color_adjustment;
            }
            MoveFlag::Castle => { // src is king's origin square, dst is king's destination square
                new_context.castling_info &= !0b00001100 >> castling_color_adjustment;
                
                let is_king_side = src & (1u64 << (self.turn as u64 * 7 * 8)) != 0;
                let is_queen_side = !is_king_side;
                
                let rook_src = 1u64 << (self.turn as u64 * (((7 * 8 + 7) * is_king_side as u64) | ((7 * 8) * is_queen_side as u64)));
                let rook_dst = 1u64 << (self.turn as u64 * (((7 * 8 + 5) * is_king_side as u64) | ((7 * 8 + 3) * is_queen_side as u64)));
                let rook_src_dst = rook_src | rook_dst;
                self.board.bb_by_color[self.turn as usize] ^= rook_src_dst;
                self.board.bb_by_piece_type[PieceType::Rook as usize] ^= rook_src_dst;
            }
            MoveFlag::PromoteToQueen => {
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Queen as usize] |= dst;
            }
            MoveFlag::PromoteToKnight => {
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Knight as usize] |= dst;
            }
            MoveFlag::PromoteToRook => {
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Rook as usize] |= dst;
            }
            MoveFlag::PromoteToBishop => {
                self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src;
                self.board.bb_by_piece_type[PieceType::Bishop as usize] |= dst;
            }
            _ => {
                panic!("invalid move flag");
            }
        }
        // update data members
        self.halfmove += 1;
        self.turn = self.turn.flip();
        self.context = Box::new(new_context);
        self.in_check = self.board.is_in_check(self.turn);
        
        if self.board.are_both_sides_insufficient_material() {
            self.termination = Some(Termination::InsufficientMaterial);
        }
        else if self.context.halfmove_clock == 100 { // fifty move rule
            self.termination = Some(Termination::FiftyMoveRule);
        }
        else {
            // update Zobrist table
            let zobrist_hash = self.board.zobrist_hash();
            let position_count = self.position_count.entry(zobrist_hash).or_insert(0);
            *position_count += 1;

            // check for repetition
            if *position_count == 3 {
                self.termination = Some(Termination::ThreefoldRepetition);
            }
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
    
    pub fn is_valid(&self) -> bool {
        // todo
        true
    }
}