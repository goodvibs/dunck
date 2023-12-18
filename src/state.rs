use std::collections::HashMap;
use crate::board::Board;
use crate::r#move::*;
use crate::utils::*;
use crate::attacks::*;
use crate::consts::*;
use crate::masks::*;
use crate::preload::ZOBRIST_TABLE;

#[derive(Clone)]
pub enum Termination {
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule
}

#[derive(Clone)]
pub struct State {
    pub board: Board,
    pub wk_castle: bool,
    pub wq_castle: bool,
    pub bk_castle: bool,
    pub bq_castle: bool,
    pub in_check: bool,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1,
    pub position_count: HashMap<u64, u8>,
    pub turn: Color,
    pub halfmove: u16,
    pub halfmove_clock: u8,
    pub termination: Option<Termination>
}

impl State {
    pub fn initial() -> State {
        let board = Board::initial();
        let position_count: HashMap<u64, u8> = HashMap::from([(board.zobrist_hash(), 1)]);
        State {
            board: board,
            wk_castle: true,
            wq_castle: true,
            bk_castle: true,
            bq_castle: true,
            in_check: false,
            double_pawn_push: -1,
            position_count: position_count,
            turn: Color::White,
            halfmove: 0,
            halfmove_clock: 0,
            termination: None
        }
    }

    pub fn from_pgn(pgn: &str) -> (State, Vec<Move>) {
        enum ParseState {
            Tag,
            MoveNum,
            Move,
            Comment,
            Annotation,
            Variation
        }

        let mut state = State::initial();
        let mut moves: Vec<Move> = Vec::new();
        let mut move_num_str = String::new();
        let mut move_str = String::new();
        let mut period_count: u16 = 0;
        let mut nest_level: u16 = 0;
        let mut parse_state = ParseState::MoveNum;
        for c in pgn.chars().chain(std::iter::once(' ')) {
            match parse_state {
                ParseState::Tag => {
                    if c == ']' {
                        parse_state = ParseState::MoveNum;
                    }
                }
                ParseState::MoveNum => {
                    if c == '{' {
                        parse_state = ParseState::Comment;
                    }
                    else if c == '[' {
                        parse_state = ParseState::Tag;
                    }
                    else if c == '(' {
                        parse_state = ParseState::Variation;
                    }
                    else if c.is_ascii_whitespace() && move_num_str.is_empty() {
                        continue;
                    }
                    else if c == '.' {
                        period_count += 1;
                        if state.turn == Color::White || period_count == 3 {
                            assert_eq!(move_num_str.parse::<u16>().unwrap(), state.halfmove / 2 + 1);
                            parse_state = ParseState::Move;
                            move_num_str.clear();
                            period_count = 0;
                        }
                    }
                    else if c.is_ascii_digit() {
                        assert_eq!(period_count, 0);
                        move_num_str.push(c);
                    }
                    else if state.turn == Color::Black {
                        move_str.push(c);
                        parse_state = ParseState::Move;
                    }
                    else {
                        panic!("invalid character in move number: {}", c);
                    }
                }
                ParseState::Move => {
                    if c.is_ascii_whitespace() {
                        if move_str.is_empty() {
                            continue;
                        }
                        let possible_moves = state.get_moves();
                        let mut matched_move: Option<Move> = None;
                        for mv in possible_moves {
                            if mv.matches(&move_str) {
                                if matched_move.is_some() {
                                    panic!("ambiguous move: {}", move_str);
                                }
                                matched_move = Some(mv);
                            }
                        }
                        match matched_move {
                            Some(mv) => {
                                moves.push(mv);
                                state.play_move(mv);
                                parse_state = ParseState::MoveNum;
                                move_str.clear();
                            },
                            None => panic!("invalid move: {}", move_str)
                        }
                    }
                    else if c == '!' || c == '?' {
                        parse_state = ParseState::Annotation;
                    }
                    else {
                        move_str.push(c);
                    }
                }
                ParseState::Comment => {
                    if c == '}' {
                        if nest_level == 0 {
                            parse_state = ParseState::MoveNum;
                        }
                        else {
                            parse_state = ParseState::Variation;
                            nest_level -= 1;
                        }
                    }
                }
                ParseState::Annotation => {
                    if c.is_ascii_whitespace() {
                        parse_state = ParseState::MoveNum;
                    }
                }
                ParseState::Variation => {
                    if c == ')' {
                        if nest_level == 0 {
                            parse_state = ParseState::MoveNum;
                        }
                        else {
                            nest_level -= 1;
                        }
                    }
                    else if c == '(' {
                        nest_level += 1;
                    }
                    else if c == '{' {
                        nest_level += 1;
                        parse_state = ParseState::Comment;
                    }
                }
            }
        }
        (state, moves)
    }

    pub fn get_moves(&self) -> Vec<Move> {
        if self.termination.is_some() {
            return Vec::with_capacity(0);
        }
        let white_occ = self.board.white();
        let black_occ = self.board.black();
        let all_occ = white_occ | black_occ;
        let mut moves: Vec<Move> = Vec::new();
        match self.turn {
            Color::White => {
                // king moves
                let king_src = self.board.wk;
                let mut black_attacks = knight_attacks(self.board.bn) |
                    king_attacks(self.board.bk) |
                    pawn_attacks(self.board.bp, Color::Black);
                for src in unpack_bb(self.board.bb) {
                    black_attacks |= bishop_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.br) {
                    black_attacks |= rook_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.bq) {
                    black_attacks |= bishop_attacks(src, all_occ) | rook_attacks(src, all_occ);
                }
                let king_moves = king_attacks(king_src) & !white_occ & !black_attacks;
                for dst in unpack_bb(king_moves) {
                    moves.push(Move::new(king_src.leading_zeros(), dst.leading_zeros(), KING_MOVE_FLAG));
                }
                if black_attacks & king_src == 0 { // if not in check
                    if self.wk_castle && ((white_occ | black_attacks) & WHITE_CASTLE_SHORT == 0) {
                        moves.push(Move::new(king_src.leading_zeros(), (king_src >> 2).leading_zeros(), CASTLE_FLAG));
                    }
                    if self.wq_castle && ((white_occ | black_attacks) & WHITE_CASTLE_LONG == 0) {
                        moves.push(Move::new(king_src.leading_zeros(), (king_src << 2).leading_zeros(), CASTLE_FLAG));
                    }
                }
                // knight moves
                let knight_srcs = unpack_bb(self.board.wn);
                for src in knight_srcs {
                    let knight_moves = knight_attacks(src) & !white_occ;
                    for dst in unpack_bb(knight_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.wn ^= src | dst;
                        if !validation_board.is_in_check(Color::White) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), KNIGHT_MOVE_FLAG));
                        }
                    }
                }
                // bishop moves
                let bishop_srcs = unpack_bb(self.board.wb);
                for src in bishop_srcs {
                    let bishop_moves = bishop_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(bishop_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.wb ^= src | dst;
                        if !validation_board.is_in_check(Color::White) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), BISHOP_MOVE_FLAG));
                        }
                    }
                }
                // rook moves
                let rook_srcs = unpack_bb(self.board.wr);
                for src in rook_srcs {
                    let rook_moves = rook_attacks(src, all_occ) & !white_occ;
                    for dst in unpack_bb(rook_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.wr ^= src | dst;
                        if !validation_board.is_in_check(Color::White) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), ROOK_MOVE_FLAG));
                        }
                    }
                }
                // queen moves
                let queen_srcs = unpack_bb(self.board.wq);
                for src in queen_srcs {
                    let queen_moves = (rook_attacks(src, all_occ) | bishop_attacks(src, all_occ)) & !white_occ;
                    for dst in unpack_bb(queen_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.wq ^= src | dst;
                        if !validation_board.is_in_check(Color::White) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), QUEEN_MOVE_FLAG));
                        }
                    }
                }
                // pawn pushes
                let pawn_srcs = unpack_bb(self.board.wp);
                for src in pawn_srcs.iter() {
                    let single_move_dst = pawn_moves(*src, Color::White) & !all_occ;
                    if single_move_dst == 0 {
                        continue;
                    }
                    // double moves
                    if single_move_dst & RANK_3 != 0 {
                        let double_move_dst = pawn_moves(single_move_dst, Color::White) & !all_occ;
                        if double_move_dst != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.wp ^= *src | double_move_dst;
                            if !validation_board.is_in_check(Color::White) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), double_move_dst.leading_zeros(), PAWN_DOUBLE_MOVE_FLAG));
                            }
                        }
                    }
                    else if single_move_dst & RANK_8 != 0 { // promotion
                        let mut validation_board = self.board.clone();
                        validation_board.wp ^= *src | single_move_dst;
                        if !validation_board.is_in_check(Color::White) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                            continue;
                        }
                    }
                    let mut validation_board = self.board.clone();
                    validation_board.wp ^= *src | single_move_dst;
                    if !validation_board.is_in_check(Color::White) { // if not moving into check
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PAWN_MOVE_FLAG));
                    }
                }
                // pawn captures
                for src in pawn_srcs {
                    let captures = pawn_attacks(src, Color::White) & black_occ;
                    for dst in unpack_bb(captures) {
                        if dst & RANK_8 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.wp ^= src | dst;
                            if !validation_board.is_in_check(Color::White) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                            }
                        }
                        else {
                            let mut validation_board = self.board.clone();
                            validation_board.wp ^= src | dst;
                            if !validation_board.is_in_check(Color::White) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PAWN_MOVE_FLAG));
                            }
                        }
                    }
                }
                // en passant
                if self.double_pawn_push != -1 {
                    if self.double_pawn_push != 0 {
                        let left_mask = FILE_A >> self.double_pawn_push - 1;
                        if self.board.wp & left_mask & RANK_5 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.wp ^= left_mask & RANK_5;
                            if !validation_board.is_in_check(Color::White) { // if not moving into check
                                moves.push(Move::new((24 + self.double_pawn_push - 1) as u32, (16 + self.double_pawn_push) as u32, EN_PASSANT_FLAG));
                            }
                        }
                    }
                    if self.double_pawn_push != 7 {
                        let right_mask = FILE_A >> self.double_pawn_push + 1;
                        if self.board.wp & right_mask & RANK_5 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.wp ^= right_mask & RANK_5;
                            if !validation_board.is_in_check(Color::White) { // if not moving into check
                                moves.push(Move::new((24 + self.double_pawn_push + 1) as u32, (16 + self.double_pawn_push) as u32, EN_PASSANT_FLAG));
                            }
                        }
                    }
                }
            },
            Color::Black => {
                // king moves
                let king_src = self.board.bk;
                let mut white_attacks = knight_attacks(self.board.wn) |
                    king_attacks(self.board.wk) |
                    pawn_attacks(self.board.wp, Color::White);
                for src in unpack_bb(self.board.wb) {
                    white_attacks |= bishop_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.wr) {
                    white_attacks |= rook_attacks(src, all_occ);
                }
                for src in unpack_bb(self.board.wq) {
                    white_attacks |= bishop_attacks(src, all_occ) | rook_attacks(src, all_occ);
                }
                let king_moves = king_attacks(king_src) & !black_occ & !white_attacks;
                for dst in unpack_bb(king_moves) {
                    moves.push(Move::new(king_src.leading_zeros(), dst.leading_zeros(), KING_MOVE_FLAG));
                }
                if white_attacks & king_src == 0 { // if not in check
                    if self.bk_castle && ((black_occ | white_attacks) & BLACK_CASTLE_SHORT == 0) {
                        moves.push(Move::new(king_src.leading_zeros(), (king_src >> 2).leading_zeros(), CASTLE_FLAG));
                    }
                    if self.bq_castle && ((black_occ | white_attacks) & BLACK_CASTLE_LONG == 0) {
                        moves.push(Move::new(king_src.leading_zeros(), (king_src << 2).leading_zeros(), CASTLE_FLAG));
                    }
                }
                // knight moves
                let knight_srcs = unpack_bb(self.board.bn);
                for src in knight_srcs {
                    let knight_moves = knight_attacks(src) & !black_occ;
                    for dst in unpack_bb(knight_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.bn ^= src | dst;
                        if !validation_board.is_in_check(Color::Black) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), KNIGHT_MOVE_FLAG));
                        }
                    }
                }
                // bishop moves
                let bishop_srcs = unpack_bb(self.board.bb);
                for src in bishop_srcs {
                    let bishop_moves = bishop_attacks(src, all_occ) & !black_occ;
                    for dst in unpack_bb(bishop_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.bb ^= src | dst;
                        if !validation_board.is_in_check(Color::Black) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), BISHOP_MOVE_FLAG));
                        }
                    }
                }
                // rook moves
                let rook_srcs = unpack_bb(self.board.br);
                for src in rook_srcs {
                    let rook_moves = rook_attacks(src, all_occ) & !black_occ;
                    for dst in unpack_bb(rook_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.br ^= src | dst;
                        if !validation_board.is_in_check(Color::Black) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), ROOK_MOVE_FLAG));
                        }
                    }
                }
                // queen moves
                let queen_srcs = unpack_bb(self.board.bq);
                for src in queen_srcs {
                    let queen_moves = (rook_attacks(src, all_occ) | bishop_attacks(src, all_occ)) & !black_occ;
                    for dst in unpack_bb(queen_moves) {
                        let mut validation_board = self.board.clone();
                        validation_board.bq ^= src | dst;
                        if !validation_board.is_in_check(Color::Black) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), QUEEN_MOVE_FLAG));
                        }
                    }
                }
                // pawn pushes
                let pawn_srcs = unpack_bb(self.board.bp);
                for src in pawn_srcs.iter() {
                    let single_move_dst = pawn_moves(*src, Color::Black) & !all_occ;
                    if single_move_dst == 0 {
                        continue;
                    }
                    // double moves
                    if single_move_dst & RANK_6 != 0 {
                        let double_move_dst = pawn_moves(single_move_dst, Color::Black) & !all_occ;
                        if double_move_dst != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.bp ^= src | double_move_dst;
                            if !validation_board.is_in_check(Color::Black) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), double_move_dst.leading_zeros(), PAWN_DOUBLE_MOVE_FLAG));
                            }
                        }
                    }
                    else if single_move_dst & RANK_1 != 0 { // promotion
                        let mut validation_board = self.board.clone();
                        validation_board.bp ^= src | single_move_dst;
                        if !validation_board.is_in_check(Color::Black) { // if not moving into check
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                            moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                        }
                        continue;
                    }
                    let mut validation_board = self.board.clone();
                    validation_board.bp ^= src | single_move_dst;
                    if !validation_board.is_in_check(Color::Black) { // if not moving into check
                        moves.push(Move::new(src.leading_zeros(), single_move_dst.leading_zeros(), PAWN_MOVE_FLAG));
                    }
                }
                // pawn captures
                for src in pawn_srcs {
                    let captures = pawn_attacks(src, Color::Black) & white_occ;
                    for dst in unpack_bb(captures) {
                        if dst & RANK_1 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.bp ^= src | dst;
                            if !validation_board.is_in_check(Color::Black) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_QUEEN_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_KNIGHT_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_ROOK_FLAG));
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PROMOTE_TO_BISHOP_FLAG));
                            }
                        }
                        else {
                            let mut validation_board = self.board.clone();
                            validation_board.bp ^= src | dst;
                            if !validation_board.is_in_check(Color::Black) { // if not moving into check
                                moves.push(Move::new(src.leading_zeros(), dst.leading_zeros(), PAWN_MOVE_FLAG));
                            }
                        }
                    }
                }
                // en passant
                if self.double_pawn_push != -1 {
                    if self.double_pawn_push != 0 { // if not on file A
                        let left_mask = FILE_A >> self.double_pawn_push - 1;
                        if self.board.bp & left_mask & RANK_4 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.bp ^= left_mask & RANK_4;
                            if !validation_board.is_in_check(Color::Black) { // if not moving into check
                                moves.push(Move::new((32 + self.double_pawn_push - 1) as u32, (40 + self.double_pawn_push) as u32, EN_PASSANT_FLAG));
                            }
                        }
                    }
                    if self.double_pawn_push != 7 { // if not on file H
                        let right_mask = FILE_A >> self.double_pawn_push + 1;
                        if self.board.bp & right_mask & RANK_4 != 0 {
                            let mut validation_board = self.board.clone();
                            validation_board.bp ^= right_mask & RANK_4;
                            if !validation_board.is_in_check(Color::Black) { // if not moving into check
                                moves.push(Move::new((32 + self.double_pawn_push + 1) as u32, (40 + self.double_pawn_push) as u32, EN_PASSANT_FLAG));
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    pub fn play_move(&mut self, mv: Move) {
        let (src_sq, dst_sq, flag) = mv.unpack();
        let src = 1 << (63 - src_sq);
        let dst = 1 << (63 - dst_sq);
        let src_dst = src | dst;
        self.halfmove += 1;
        match self.turn {
            Color::White => {
                if flag >= PAWN_MOVE_FLAG || self.board.bp & dst != 0 || self.board.bn & dst != 0 || self.board.bb & dst != 0 || self.board.br & dst != 0 || self.board.bq & dst != 0 || self.board.bk & dst != 0 {
                    self.halfmove_clock = 0;
                    self.board.bp &= !dst;
                    self.board.bn &= !dst;
                    self.board.bb &= !dst;
                    self.board.br &= !dst;
                    self.board.bq &= !dst;
                }
                else {
                    self.halfmove_clock += 1;
                }
                if flag != PAWN_DOUBLE_MOVE_FLAG {
                    self.double_pawn_push = -1;
                }
                match flag {
                    KNIGHT_MOVE_FLAG => {
                        self.board.wn ^= src_dst;
                    },
                    BISHOP_MOVE_FLAG => {
                        self.board.wb ^= src_dst;
                    },
                    ROOK_MOVE_FLAG => {
                        self.board.wr ^= src_dst;
                    },
                    QUEEN_MOVE_FLAG => {
                        self.board.wq ^= src_dst;
                    },
                    KING_MOVE_FLAG => {
                        self.board.wk ^= src_dst;
                    },
                    PAWN_MOVE_FLAG | EN_PASSANT_FLAG => {
                        self.board.wp ^= src_dst;
                    },
                    PAWN_DOUBLE_MOVE_FLAG => {
                        self.board.wp ^= src_dst;
                        self.double_pawn_push = (src_sq % 8) as i8;
                    },
                    PROMOTE_TO_QUEEN_FLAG => {
                        self.board.wq |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_KNIGHT_FLAG => {
                        self.board.wn |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_ROOK_FLAG => {
                        self.board.wr |= dst;
                        self.board.wp &= !src;
                    },
                    PROMOTE_TO_BISHOP_FLAG => {
                        self.board.wb |= dst;
                        self.board.wp &= !src;
                    },
                    CASTLE_FLAG => {
                        self.board.wk &= !0x08;
                        if dst == src >> 2 { // short castle
                            self.board.wr &= !0x01;
                            self.board.wr |= 0x04;
                            self.board.wk |= 0x02;
                            self.wk_castle = false;
                        }
                        else if dst == src << 2 { // long castle
                            self.board.wr &= !0x80;
                            self.board.wr |= 0x10;
                            self.board.wk |= 0x20;
                            self.wq_castle = false;
                        }
                    },
                    _ => {
                        panic!("invalid move flag");
                    }
                }
                self.turn = Color::Black;
            },
            Color::Black => {
                if flag >= PAWN_MOVE_FLAG || self.board.wp & dst != 0 || self.board.wn & dst != 0 || self.board.wb & dst != 0 || self.board.wr & dst != 0 || self.board.wq & dst != 0 || self.board.wk & dst != 0 {
                    self.halfmove_clock = 0;
                    self.board.wp &= !dst;
                    self.board.wn &= !dst;
                    self.board.wb &= !dst;
                    self.board.wr &= !dst;
                    self.board.wq &= !dst;
                }
                else {
                    self.halfmove_clock += 1;
                }
                if flag != PAWN_DOUBLE_MOVE_FLAG {
                    self.double_pawn_push = -1;
                }
                match flag {
                    KNIGHT_MOVE_FLAG => {
                        self.board.bn ^= src_dst;
                    },
                    BISHOP_MOVE_FLAG => {
                        self.board.bb ^= src_dst;
                    },
                    ROOK_MOVE_FLAG => {
                        self.board.br ^= src_dst;
                    },
                    QUEEN_MOVE_FLAG => {
                        self.board.bq ^= src_dst;
                    },
                    KING_MOVE_FLAG => {
                        self.board.bk ^= src_dst;
                    },
                    PAWN_MOVE_FLAG | EN_PASSANT_FLAG => {
                        self.board.bp ^= src_dst;
                    },
                    PAWN_DOUBLE_MOVE_FLAG => {
                        self.board.bp ^= src_dst;
                        self.double_pawn_push = (src_sq % 8) as i8;
                    },
                    PROMOTE_TO_QUEEN_FLAG => {
                        self.board.bq |= dst;
                        self.board.bp &= !src;
                    },
                    PROMOTE_TO_KNIGHT_FLAG => {
                        self.board.bn |= dst;
                        self.board.bp &= !src;
                    },
                    PROMOTE_TO_ROOK_FLAG => {
                        self.board.br |= dst;
                        self.board.bp &= !src;
                    },
                    PROMOTE_TO_BISHOP_FLAG => {
                        self.board.bb |= dst;
                        self.board.bp &= !src;
                    },
                    CASTLE_FLAG => {
                        self.board.bk &= !(0x08 << 56);
                        if dst == src >> 2 { // short castle
                            self.board.br &= !(0x01 << 56);
                            self.board.br |= 0x04 << 56;
                            self.board.bk |= 0x02 << 56;
                            self.bk_castle = false;
                        }
                        else if dst == src << 2 { // long castle
                            self.board.br &= !(0x80 << 56);
                            self.board.br |= 0x10 << 56;
                            self.board.bk |= 0x20 << 56;
                            self.bq_castle = false;
                        }
                    },
                    _ => {
                        panic!("invalid move flag");
                    }
                }
                self.turn = Color::White;
            }
        }
        let zobrist_hash = self.board.zobrist_hash();
        let count = self.position_count.entry(zobrist_hash).or_insert(0);
        *count += 1;
        if *count == 3 {
            self.termination = Some(Termination::ThreefoldRepetition);
        }
        else if self.halfmove_clock == 100 {
            self.termination = Some(Termination::FiftyMoveRule);
        }
    }
}