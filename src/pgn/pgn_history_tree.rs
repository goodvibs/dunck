use std::collections::HashMap;
use std::fmt::{Debug, Display};
use indexmap::IndexMap;
use crate::pgn::pgn_move_node::PgnMoveNode;
use crate::r#move::Move;
use crate::state::State;

#[derive(Eq)]
pub struct PgnHistoryTree {
    pub tags: IndexMap<String, String>,
    pub initial_state: State,
    pub head: Option<*mut PgnMoveNode>,
}

#[derive(Debug)]
pub enum PgnParseState {
    InitialState,
    ParsingTag,
    ParsingMoveNumberOrSomethingElse,
    ParsingMove,
    ParsingComment,
    ParsingAnnotation,
    ParsingNag
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

impl PgnHistoryTree {
    fn check_and_add_tag(&mut self, tag: &str) {
        // todo!();
    }

    pub fn from_pgn(pgn: &str) -> Result<PgnHistoryTree, PgnParseError> {
        let mut pgn_history_tree: PgnHistoryTree = PgnHistoryTree {
            tags: IndexMap::new(),
            initial_state: State::initial(),
            head: None
        };
        
        let mut parse_state = PgnParseState::InitialState;
        let mut tail_node: Option<*mut PgnMoveNode> = None;
        let mut current_state = State::initial();
        let mut previous_state = State::blank();
        
        // for variations
        let mut current_state_and_tail_node_stack: Vec<(State, *mut PgnMoveNode)> = Vec::new();

        // for building string values
        let mut tag_builder = String::new();
        let mut move_number_builder = String::new();
        let mut move_san_builder = String::new();

        for (i, c) in pgn.chars().chain(std::iter::once(' ')).enumerate() {
            match parse_state {
                PgnParseState::InitialState => {
                    match c {
                        '[' => {
                            parse_state = PgnParseState::ParsingTag;
                        },
                        _ if c.is_ascii_whitespace() => {
                            continue;
                        },
                        _ if c.is_ascii_digit() => {
                            move_number_builder.push(c);
                            parse_state = PgnParseState::ParsingMoveNumberOrSomethingElse;
                        },
                        _ => {
                            return Err(PgnParseError::UnexpectedCharacter(PgnParseState::InitialState, pgn[..i+1].to_string()));
                        }
                    }
                }
                PgnParseState::ParsingTag => {
                    match c {
                        ']' => {
                            pgn_history_tree.check_and_add_tag(&tag_builder);
                            tag_builder.clear();
                            parse_state = PgnParseState::InitialState;
                        },
                        _ => {
                            tag_builder.push(c);
                        }
                    }
                }
                PgnParseState::ParsingMoveNumberOrSomethingElse => {
                    let expected_number = current_state.halfmove / 2 + 1;
                    match c {
                        '{' => {
                            parse_state = PgnParseState::ParsingComment;
                        },
                        '(' => {
                            match tail_node {
                                Some(tail_node_unwrapped) => {
                                    current_state_and_tail_node_stack.push((current_state.clone(), tail_node_unwrapped));
                                    current_state = previous_state.clone();
                                },
                                None => return Err(PgnParseError::UnexpectedVariation(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[i..].to_string()))
                            }
                        },
                        ')' => {
                            match current_state_and_tail_node_stack.pop() {
                                Some((new_current_state, tail_node_unwrapped)) => {
                                    current_state = new_current_state;
                                    tail_node = Some(tail_node_unwrapped);
                                }
                                None => return Err(PgnParseError::UnfinishedVariation(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()))
                            }
                        },
                        '$' => {
                            parse_state = PgnParseState::ParsingNag;
                        },
                        '.' => {
                            let move_number_parse_result = move_number_builder.parse::<u16>();
                            match move_number_parse_result {
                                Ok(move_number) => {
                                    if move_number != expected_number {
                                        return Err(PgnParseError::BadMoveNumber(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()));
                                    }
                                },
                                Err(_) => return Err(PgnParseError::UnexpectedValue(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()))
                            }
                            move_number_builder.clear();
                            parse_state = PgnParseState::ParsingMove;
                        },
                        _ if c.is_ascii_whitespace() && move_number_builder.is_empty() => {
                            continue;
                        },
                        _ if c.is_ascii_digit() && (c != '0' || !move_number_builder.is_empty()) => {
                            move_number_builder.push(c);
                        },
                        _ if c.is_ascii_alphabetic() || c == '0' => {
                            move_san_builder.push(c);
                            parse_state = PgnParseState::ParsingMove;
                        },
                        _ => {
                            return Err(PgnParseError::UnexpectedCharacter(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()));
                        }
                    }
                }
                PgnParseState::ParsingMove => {
                    match c {
                        '.' => {
                            continue;
                        },
                        '!'|'?' => {
                            parse_state = PgnParseState::ParsingAnnotation;
                        },
                        _ if c.is_ascii_whitespace() => {
                            if move_san_builder.is_empty() {
                                continue;
                            }
                            let possible_moves = current_state.get_moves();
                            let mut matched_move: Option<Move> = None;
                            for mv in possible_moves {
                                if mv.matches(&move_san_builder) {
                                    if matched_move.is_some() {
                                        return Err(PgnParseError::AmbiguousMove(PgnParseState::ParsingMove, pgn[..i+1].to_string()));
                                    }
                                    matched_move = Some(mv);
                                }
                            }
                            match matched_move {
                                Some(mv) => {
                                    previous_state = current_state.clone();
                                    current_state.play_move(mv);
                                    let new_node = PgnMoveNode::new(mv, current_state.clone(), tail_node);
                                    match tail_node {
                                        Some(previous_node_unwrapped) => unsafe {
                                            (*previous_node_unwrapped).next_nodes.push(new_node);
                                        },
                                        None => pgn_history_tree.head = Some(new_node)
                                    }
                                    tail_node = Some(new_node);
                                },
                                None => return Err(PgnParseError::BadMove(PgnParseState::ParsingMove, pgn[..i+1].to_string()))
                            }
                            parse_state = PgnParseState::ParsingMoveNumberOrSomethingElse;
                            move_san_builder.clear();
                        }
                        _ => {
                            move_san_builder.push(c);
                        }
                    }
                }
                PgnParseState::ParsingComment => {
                    if c == '}' {
                        parse_state = PgnParseState::ParsingMoveNumberOrSomethingElse;
                    }
                }
                PgnParseState::ParsingAnnotation => {
                    if c.is_ascii_whitespace() {
                        parse_state = PgnParseState::ParsingMoveNumberOrSomethingElse;
                    }
                }
                PgnParseState::ParsingNag => {
                    if c.is_ascii_whitespace() {
                        parse_state = PgnParseState::ParsingMoveNumberOrSomethingElse;
                    }
                }
            }
        }
        Ok(pgn_history_tree)
    }

    fn tags_pgn(&self) -> String {
        let mut res = String::new();
        for tag in self.tags.iter() {
            res += &*format!("[{} \"{}\"]\n", tag.0, tag.1);
        }
        res
    }

    fn pgn_helper(&self, should_render_variations: bool) -> String {
        let mut res = String::new();
        if let Some(head) = self.head {
            unsafe {
                res += &*format!("{}", (*head).pgn(self.initial_state.clone(), should_render_variations, false, 0));
            }
        }
        res
    }

    pub fn pgn(&self) -> String {
        format!("{}\n{}", self.tags_pgn(), self.pgn_helper(true))
    }

    pub fn main_line_pgn(&self) -> String {
        self.pgn_helper(false)
    }

    pub fn main_line_moves(&self) -> Vec<Move> {
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

impl Drop for PgnHistoryTree {
    fn drop(&mut self) {
        if let Some(head) = self.head {
            unsafe {
                drop(Box::from_raw(head));
            }
        }
    }
}

impl Display for PgnHistoryTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pgn())
    }
}

impl Debug for PgnHistoryTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tags_pgn())
    }
}

impl PartialEq<Self> for PgnHistoryTree {
    fn eq(&self, other: &Self) -> bool {
        if self.initial_state != other.initial_state {
            return false;
        }
        if self.head.is_some() && other.head.is_some() {
            unsafe {
                return *self.head.unwrap() == *other.head.unwrap();
            }
        }
        else if self.head.is_some() || other.head.is_some() {
            return false;
        }
        true
    }
}