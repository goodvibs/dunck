use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use indexmap::IndexMap;
use crate::pgn::pgn_move_tree_node::PgnMoveTreeNode;
use crate::pgn::PgnMoveTreeTraverser;
use crate::r#move::Move;
use crate::state::State;

pub struct PgnMoveTree {
    pub tags: IndexMap<String, String>,
    pub head: *mut PgnMoveTreeNode,
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

#[derive(Debug)]
pub enum PgnParseError {
    UnexpectedValue(PgnParseState, String),
    WrongMoveNumber(PgnParseState, String),
    AmbiguousMove(PgnParseState, String),
    IllegalMove(PgnParseState, String),
    IllegalVariationStart(PgnParseState, String),
    UnfinishedVariation(PgnParseState, String),
    UnfinishedComment(PgnParseState, String)
}

impl Display for PgnParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnParseError::UnexpectedValue(state, parsed) => write!(f, "PGN Parse Error: 'Unexpected value' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::WrongMoveNumber(state, parsed) => write!(f, "PGN Parse Error: 'Wrong move number' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::AmbiguousMove(state, parsed) => write!(f, "PGN Parse Error: 'Ambiguous move' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::IllegalMove(state, parsed) => write!(f, "PGN Parse Error: 'Illegal move' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::IllegalVariationStart(state, parsed) => write!(f, "PGN Parse Error: 'Illegal variation start' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnfinishedVariation(state, parsed) => write!(f, "PGN Parse Error: 'Unfinished variation' at state '{:?}'\nParsed:\n'{}'", state, parsed),
            PgnParseError::UnfinishedComment(state, parsed) => write!(f, "PGN Parse Error: 'Unfinished comment' at state '{:?}'\nParsed:\n'{}'", state, parsed)
        }
    }
}

impl Error for PgnParseError {}

impl PgnMoveTree {
    fn check_and_add_tag(&mut self, tag: &str) {
        // todo!();
    }

    pub fn from_pgn(pgn: &str) -> Result<PgnMoveTree, PgnParseError> {
        let mut pgn_history_tree: PgnMoveTree = PgnMoveTree {
            tags: IndexMap::new(),
            head: PgnMoveTreeNode::new_raw_linked_to_previous(None, "".to_string(), None, State::initial())
        };

        let mut parse_state = PgnParseState::InitialState;
        let mut tail_node: *mut PgnMoveTreeNode = pgn_history_tree.head;
        let mut current_state = State::initial();
        let mut previous_state = State::blank();

        // for variations
        let mut current_state_and_tail_node_stack: Vec<(State, *mut PgnMoveTreeNode)> = Vec::new();

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
                            return Err(PgnParseError::UnexpectedValue(PgnParseState::InitialState, pgn[..i+1].to_string()));
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
                        '(' => unsafe {
                            match (*tail_node).move_and_san_and_previous_node {
                                Some((_, _, node)) => {
                                    current_state_and_tail_node_stack.push((current_state.clone(), tail_node));
                                    current_state = previous_state.clone();
                                    // tail_node = node;
                                },
                                None => return Err(PgnParseError::IllegalVariationStart(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[i..].to_string()))
                            }
                        },
                        ')' => {
                            match current_state_and_tail_node_stack.pop() {
                                Some((new_current_state, new_tail_node)) => {
                                    current_state = new_current_state;
                                    tail_node = new_tail_node;
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
                                        return Err(PgnParseError::WrongMoveNumber(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()));
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
                            return Err(PgnParseError::UnexpectedValue(PgnParseState::ParsingMoveNumberOrSomethingElse, pgn[..i+1].to_string()));
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
                            if move_san_builder == "Bxb7" {
                                println!("{:?}", &possible_moves);
                                current_state.board.print();
                            }
                            let mut matched_move: Option<Move> = None;
                            for mv in &possible_moves {
                                if mv.matches(&move_san_builder) {
                                    if matched_move.is_some() {
                                        return Err(PgnParseError::AmbiguousMove(PgnParseState::ParsingMove, pgn[..i+1].to_string()));
                                    }
                                    matched_move = Some(*mv);
                                }
                            }
                            match matched_move {
                                Some(mv) => {
                                    previous_state = current_state.clone();
                                    current_state.play_move(mv);
                                    let san = mv.san(&previous_state, &current_state, &possible_moves);
                                    let new_node = PgnMoveTreeNode::new_raw_linked_to_previous(Some(mv), san, Some(tail_node), current_state.clone());
                                    tail_node = new_node;
                                },
                                None => return Err(PgnParseError::IllegalMove(PgnParseState::ParsingMove, pgn[..i+1].to_string()))
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
        unsafe {
            res += &*format!("{}", (*self.head).pgn(should_render_variations, true, 0));
        }
        res
    }

    pub fn pgn(&self) -> String {
        if self.tags.is_empty() {
            return self.pgn_helper(true);
        }
        format!("{}\n{}", self.tags_pgn(), self.pgn_helper(true))
    }

    pub fn main_line_pgn(&self) -> String {
        self.pgn_helper(false)
    }

    // pub fn main_line_moves(&self) -> Vec<Move> {
    //     let mut res: Vec<Move> = Vec::new();
    //     unsafe {
    //         match (*self.head).move_and_san_and_previous_node {
    //             None => (),
    //             Some((mv, _, _)) => { 
    //                 res.push(mv);
    //                 while let Some(next_node) = (*self.head).next_main_node() {
    //                     res.push((*next_node).move_and_san_and_previous_node.unwrap().0);
    //                 }
    //             }
    //         }
    //     }
    //     
    //     res
    // }
    
    pub fn traverser(&self) -> PgnMoveTreeTraverser {
        PgnMoveTreeTraverser::new(self)
    }
}

impl Drop for PgnMoveTree {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.head));
        }
    }
}

impl Display for PgnMoveTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pgn())
    }
}

impl Debug for PgnMoveTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tags_pgn())
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    fn load_input_and_expected_pgn(file_name: &str) -> (String, String) {
        let input_pgn = fs::read_to_string(format!("src/pgn/test_pgn_files/{}.pgn", file_name)).expect("Could not read file");
        let expected_pgn = fs::read_to_string(format!("src/pgn/test_pgn_files/{}_formatted.pgn", file_name)).expect("Could not read file");
        (input_pgn, expected_pgn)
    }
    
    fn test_pgn(input_pgn: &str, expected_pgn: &str) {
        let pgn_tree = PgnMoveTree::from_pgn(input_pgn).unwrap();
        assert_eq!(pgn_tree.pgn(), expected_pgn);
    }
    
    fn generic_pgn_test(file_name: &str) {
        let (input_pgn, expected_pgn) = load_input_and_expected_pgn(file_name);
        test_pgn(&input_pgn, &expected_pgn);
    }
    
    // #[test]
    // fn empty_pgn_test() {
    //     let input_pgn = "";
    //     let pgn_tree = PgnMoveTree::from_pgn(input_pgn).unwrap();
    //     assert!(pgn_tree.tags.is_empty());
    //     unsafe { 
    //         assert!((*pgn_tree.head).move_and_san_and_previous_node.is_none());
    //     }
    //     assert_eq!(pgn_tree.pgn(), "");
    // }
    // 
    // #[test]
    // fn complex_pgn_test() {
    //     generic_pgn_test("complex");
    // }
    // 
    // #[test]
    // fn rosen1_pgn_test() {
    //     generic_pgn_test("rosen1");
    // }
}