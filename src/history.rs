use std::string::ParseError;
use crate::r#move::Move;
use crate::state::State;
use crate::utils::Color;

pub struct HistoryNode {
    pub moves: Vec<Move>,
    pub final_state: State,
    pub prev_node: Option<*mut HistoryNode>,
    pub next_nodes: Vec<*mut HistoryNode>
}

impl HistoryNode {
    fn new(moves: Vec<Move>, final_state: State, prev_node: Option<*mut HistoryNode>) -> *mut HistoryNode {
        &mut HistoryNode {
            moves,
            final_state,
            prev_node,
            next_nodes: Vec::new()
        }
    }

    fn next_main(&self) -> Option<&*mut HistoryNode> {
        self.next_nodes.last()
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
    Annotation
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

        let mut state = State::initial();
        let mut last_state = State::initial();
        let mut moves: Vec<Move> = Vec::new();
        let mut prev_node: Option<*mut HistoryNode> = None;
        let mut variation_nest_level: u16 = 0;

        let mut tag = String::new();
        let mut move_num_str = String::new();
        let mut move_str = String::new();

        let mut parse_state = PgnParseState::Initial;
        for c in pgn.chars().chain(std::iter::once(' ')) {
            print!("{}", c);
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
                PgnParseState::MoveNum => unsafe {
                    let expected_number = state.halfmove / 2 + 1;
                    if c == '{' {
                        parse_state = PgnParseState::Comment;
                    }
                    else if c == '(' {
                        let node = HistoryNode::new(moves.clone(), state.clone(), prev_node);
                        state = last_state.clone();
                        prev_node = Some(node);
                        if head.is_none() {
                            head = prev_node;
                        }
                        else {
                            (*(*node).prev_node.unwrap()).next_nodes.push(node);
                        }
                        variation_nest_level += 1;
                    }
                    else if c == ')' {
                        if !move_num_str.is_empty() || prev_node.is_none() {
                            return Err(PgnParseError::UnexpectedCharacter(PgnParseState::MoveNum, '('))
                        }
                        let node = HistoryNode::new(moves.clone(), state.clone(), prev_node);
                        let prev_node_unwrapped = prev_node.unwrap();
                        (*prev_node_unwrapped).next_nodes.push(node);
                        state = (*prev_node_unwrapped).final_state.clone();
                        moves = (*prev_node_unwrapped).moves.clone();
                        prev_node = (*prev_node_unwrapped).prev_node;
                        variation_nest_level -= 1;
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
                    else if state.turn == Color::Black && (c.is_ascii_alphabetic() || c == '0') {
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
                        let possible_moves = state.get_moves();
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
                                last_state = state.clone();
                                state.play_move(mv);
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
            }
        }
        Ok(History {
            tags,
            head
        })
    }
}