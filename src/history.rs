use std::any::Any;
use std::fmt::Debug;
use std::string::ParseError;
use crate::r#move::Move;
use crate::state::State;
use crate::utils::Color;

#[derive(Clone)]
pub struct MoveNode {
    pub current_move: Move,
    pub fullmove: u16,
    pub turn: Color,
    pub previous_node: Option<*mut MoveNode>,
    pub next_nodes: Vec<*mut MoveNode>
}

impl MoveNode {
    fn new(current_move: Move, fullmove: u16, turn: Color, previous_node: Option<*mut MoveNode>) -> *mut MoveNode {
        Box::into_raw(Box::new(MoveNode {
            current_move,
            fullmove,
            turn,
            previous_node,
            next_nodes: Vec::new()
        }))
    }

    fn next_main_node(&self) -> Option<*mut MoveNode> {
        self.next_nodes.last().cloned()
    }
}

impl Drop for MoveNode {
    fn drop(&mut self) {
        for node in self.next_nodes.iter() {
            unsafe {
                drop(Box::from_raw(*node));
            }
        }
    }
}

pub struct History {
    pub tags: Vec<String>,
    pub initial_state: Option<State>,
    pub head: Option<*mut MoveNode>,
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

pub enum PgnParseError {
    UnexpectedCharacter(PgnParseState, String),
    UnexpectedValue(PgnParseState, String),
    BadMoveNumber(PgnParseState, String),
    AmbiguousMove(PgnParseState, String),
    BadMove(PgnParseState, String),
    UnexpectedVariation(PgnParseState, String),
    UnfinishedVariation(PgnParseState, String),
    UnfinishedComment(PgnParseState, String)
}

impl Debug for PgnParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnParseError::UnexpectedCharacter(state, parsed) => write!(f, "PGN Parse Error: 'Unexpected character' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnexpectedValue(state, parsed) => write!(f, "PGN Parse Error: 'Unexpected value' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::BadMoveNumber(state, parsed) => write!(f, "PGN Parse Error: 'Bad move number' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::AmbiguousMove(state, parsed) => write!(f, "PGN Parse Error: 'Ambiguous move' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::BadMove(state, parsed) => write!(f, "PGN Parse Error: 'Bad move' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnexpectedVariation(state, parsed) => write!(f, "PGN Parse Error: 'Unexpected variation' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnfinishedVariation(state, parsed) => write!(f, "PGN Parse Error: 'Unfinished variation' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnfinishedComment(state, parsed) => write!(f, "PGN Parse Error: 'Unfinished comment' at state '{:?}'\nParsed:\n'{}'", state, parsed)
        }
    }
}

impl History {
    pub fn from_pgn(pgn: &str) -> Result<History, PgnParseError> {
        let mut tags: Vec<String> = Vec::new();
        let mut previous_node: Option<*mut MoveNode> = None;
        let mut head: Option<*mut MoveNode> = None;

        let mut parse_state = PgnParseState::Initial;
        let mut current_state = State::initial();
        let mut state_before_move = State::initial(); // remember state before move in case of variation
        let mut state_stack: Vec<(State, State)> = Vec::new(); // remember (current_state, state_before_move) before variation in case of nested variation
        let mut previous_node_stack: Vec<*mut MoveNode> = Vec::new(); // remember previous node in case of variation

        // for building string values
        let mut tag = String::new();
        let mut move_num_str = String::new();
        let mut move_str = String::new();

        for (i, c) in pgn.chars().chain(std::iter::once(' ')).enumerate() {
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
                        return Err(PgnParseError::UnexpectedCharacter(PgnParseState::Initial, pgn[..i+1].to_string()));
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
                    match c {
                        '{' => {
                            parse_state = PgnParseState::Comment;
                        },
                        '(' => {
                            if previous_node.is_none() {
                                return Err(PgnParseError::UnexpectedVariation(PgnParseState::MoveNum, pgn[i..].to_string()));
                            }
                            state_stack.push((current_state.clone(), state_before_move.clone()));
                            previous_node_stack.push(previous_node.unwrap());
                            current_state = state_before_move.clone();
                        },
                        ')' => {
                            if state_stack.is_empty() || previous_node_stack.is_empty() {
                                return Err(PgnParseError::UnfinishedVariation(PgnParseState::MoveNum, pgn[..i+1].to_string()));
                            }
                            (current_state, state_before_move) = state_stack.pop().unwrap();
                            previous_node = previous_node_stack.pop();
                        },
                        '$' => {
                            parse_state = PgnParseState::Nag;
                        },
                        '.' => {
                            let move_num_res = move_num_str.parse::<u16>();
                            if move_num_res.is_ok() {
                                let move_num = move_num_res.unwrap();
                                if move_num != expected_number {
                                    return Err(PgnParseError::BadMoveNumber(PgnParseState::MoveNum, pgn[..i+1].to_string()));
                                }
                            }
                            else {
                                return Err(PgnParseError::UnexpectedValue(PgnParseState::MoveNum, pgn[..i+1].to_string()));
                            }
                            move_num_str.clear();
                            parse_state = PgnParseState::Move;
                        },
                        _ if c.is_ascii_whitespace() && move_num_str.is_empty() => {
                            continue;
                        },
                        _ if c.is_ascii_digit() && (c != '0' || !move_num_str.is_empty()) => {
                            move_num_str.push(c);
                        },
                        _ if c.is_ascii_alphabetic() || c == '0' => {
                            move_str.push(c);
                            parse_state = PgnParseState::Move;
                        },
                        _ => {
                            return Err(PgnParseError::UnexpectedCharacter(PgnParseState::MoveNum, pgn[..i+1].to_string()));
                        }
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
                                    return Err(PgnParseError::AmbiguousMove(PgnParseState::Move, pgn[..i+1].to_string()));
                                }
                                matched_move = Some(mv);
                            }
                        }
                        match matched_move {
                            Some(mv) => {
                                state_before_move = current_state.clone();
                                current_state.play_move(mv);
                                move_str.clear();
                                let new_node = MoveNode::new(mv, current_state.get_fullmove(), current_state.turn, previous_node);
                                if previous_node.is_some() {
                                    unsafe {
                                        (*previous_node.unwrap()).next_nodes.push(new_node);
                                    }
                                }
                                else {
                                    head = Some(new_node);
                                }
                                previous_node = Some(new_node);
                            },
                            None => return Err(PgnParseError::BadMove(PgnParseState::Move, pgn[..i+1].to_string()))
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
            initial_state: None,
            head
        })
    }

    pub fn main_line(&self) -> Vec<Move> {
        let mut res: Vec<Move> = Vec::new();
        if let Some(head) = self.head {
            let mut current_node = head;
            unsafe {
                while let Some(next_node) = (*current_node).next_main_node() {
                    res.push((*current_node).current_move);
                    current_node = next_node;
                }
            }
        }
        res
    }
}

impl Drop for History {
    fn drop(&mut self) {
        if let Some(head) = self.head {
            unsafe {
                drop(Box::from_raw(head));
            }
        }
    }
}