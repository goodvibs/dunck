use std::cell::RefCell;
use std::rc::Rc;
use std::string::ParseError;
use crate::r#move::Move;
use crate::state::State;
use crate::utils::Color;

#[derive(Clone)]
pub struct HistoryNode {
    pub moves: Vec<Move>,
    pub final_state: State,
    pub prev_node: Option<*mut HistoryNode>,
    pub next_nodes: Vec<*mut HistoryNode>
}

impl HistoryNode {
    fn new(moves: Vec<Move>, final_state: State, prev_node: Option<*mut HistoryNode>) -> *mut HistoryNode {
        Box::into_raw(Box::new(HistoryNode {
            moves,
            final_state,
            prev_node,
            next_nodes: Vec::new()
        }))
    }

    fn next_main(&self) -> Option<*mut HistoryNode> {
        self.next_nodes.last().cloned()
    }
}

pub struct History {
    pub tags: Vec<String>,
    pub head: Option<*mut HistoryNode>,
}

#[derive(Debug)]
pub enum PgnParseState {
    Initial,
    Tag,
    MoveNum,
    Move,
    Comment,
    Annotation,
    Nag
}

#[derive(Debug)]
pub enum PgnParseError {
    UnexpectedCharacter(PgnParseState, char),
    UnexpectedValue(PgnParseState, String),
    BadMoveNumber(u16),
    AmbiguousMove(String),
    BadMove(String),
    UnfinishedVariation(PgnParseState),
    UnfinishedComment(String)
}

impl History {
    pub fn from_pgn(pgn: &str) -> Result<History, PgnParseError> {
        let mut tags: Vec<String> = Vec::new();
        let mut head: Option<*mut HistoryNode> = None;

        let mut current_state = State::initial();
        let mut state_before_move = State::initial(); // remember state before move in case of variation
        let mut state_stack: Vec<(State, State)> = Vec::new();
        let mut moves: Vec<Move> = Vec::new();
        let mut prev_node: Option<*mut HistoryNode> = None;
        // let mut variation_nest_level: u16 = 0;

        let mut tag = String::new();
        let mut move_num_str = String::new();
        let mut move_str = String::new();

        let mut parse_state = PgnParseState::Initial;
        for c in pgn.chars().chain(std::iter::once(' ')) {
            match parse_state {
                PgnParseState::Initial => {
                    if c.is_ascii_whitespace() {
                        continue;
                    }
                    if c == '[' {
                        parse_state = PgnParseState::Tag;
                    }
                    else if c.is_ascii_digit() {
                        move_num_str.push(c);
                        parse_state = PgnParseState::MoveNum;
                    }
                    else {
                        return Err(PgnParseError::UnexpectedCharacter(PgnParseState::Initial, c));
                    }
                }
                PgnParseState::Tag => {
                    if c == ']' {
                        tags.push(tag);
                        tag = String::new();
                        parse_state = PgnParseState::Initial;
                    }
                    else {
                        tag.push(c);
                    }
                }
                PgnParseState::MoveNum => {
                    let expected_number = current_state.halfmove / 2 + 1;
                    if c == '{' {
                        parse_state = PgnParseState::Comment;
                    }
                    else if c == '(' {
                        // TODO: create variation fork
                        let new_node = HistoryNode::new(moves.clone(), current_state.clone(), prev_node);
                        if prev_node.is_some() {
                            unsafe {
                                (*prev_node.unwrap()).next_nodes.push(new_node);
                                (*new_node).prev_node = prev_node;
                            }
                        }
                        else {
                            head = Some(new_node);
                        }
                        prev_node = Some(new_node);
                        moves.clear();
                        state_stack.push((current_state.clone(), state_before_move.clone()));
                        current_state = state_before_move.clone();
                        // variation_nest_level += 1;
                    }
                    else if c == ')' {
                        if !move_num_str.is_empty() || prev_node.is_none() || state_stack.is_empty() {
                            return Err(PgnParseError::UnexpectedCharacter(PgnParseState::MoveNum, '('))
                        }
                        // TODO: end variation fork and go back to parent
                        let new_node = HistoryNode::new(moves.clone(), current_state.clone(), prev_node);
                        unsafe {
                            (*prev_node.unwrap()).next_nodes.push(new_node);
                            (*new_node).prev_node = prev_node;
                        }
                        moves.clear();
                        (current_state, state_before_move) = state_stack.pop().unwrap();
                        // variation_nest_level -= 1;
                    }
                    else if c == '$' {
                        parse_state = PgnParseState::Nag;
                    }
                    else if c.is_ascii_whitespace() && move_num_str.is_empty() {
                        continue;
                    }
                    else if c.is_ascii_digit() && (c != '0' || !move_num_str.is_empty()) {
                        move_num_str.push(c);
                    }
                    else if c == '.' {
                        let move_num_res = move_num_str.parse::<u16>();
                        if move_num_res.is_ok() {
                            let move_num = move_num_res.unwrap();
                            if move_num != expected_number {
                                return Err(PgnParseError::BadMoveNumber(move_num));
                            }
                        }
                        else {
                            return Err(PgnParseError::UnexpectedValue(PgnParseState::MoveNum, move_num_str));
                        }
                        move_num_str.clear();
                        parse_state = PgnParseState::Move;
                    }
                    else if current_state.turn == Color::Black && (c.is_ascii_alphabetic() || c == '0') {
                        move_str.push(c);
                        parse_state = PgnParseState::Move;
                    }
                    else {
                        return Err(PgnParseError::UnexpectedCharacter(PgnParseState::MoveNum, c))
                    }
                }
                PgnParseState::Move => {
                    if c == '.' {
                        continue;
                    }
                    else if c.is_ascii_whitespace() {
                        if move_str.is_empty() {
                            continue;
                        }
                        let possible_moves = current_state.get_moves();
                        let mut matched_move: Option<Move> = None;
                        for mv in possible_moves {
                            if mv.matches(&move_str) {
                                if matched_move.is_some() {
                                    return Err(PgnParseError::AmbiguousMove(move_str));
                                }
                                matched_move = Some(mv);
                            }
                        }
                        match matched_move {
                            Some(mv) => {
                                moves.push(mv);
                                state_before_move = current_state.clone();
                                current_state.play_move(mv);
                                move_str.clear();
                            },
                            None => return Err(PgnParseError::BadMove(move_str))
                        }
                        parse_state = PgnParseState::MoveNum;
                    }
                    else if c == '!' || c == '?' {
                        parse_state = PgnParseState::Annotation;
                    }
                    else {
                        move_str.push(c);
                    }
                }
                PgnParseState::Comment => {
                    if c == '}' {
                        parse_state = PgnParseState::MoveNum;
                    }
                }
                PgnParseState::Annotation => {
                    if c.is_ascii_whitespace() {
                        parse_state = PgnParseState::MoveNum;
                    }
                }
                PgnParseState::Nag => {
                    if c.is_ascii_whitespace() {
                        parse_state = PgnParseState::MoveNum;
                    }
                }
            }
        }
        Ok(History {
            tags,
            head
        })
    }
}