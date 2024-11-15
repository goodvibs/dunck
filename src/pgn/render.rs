use crate::pgn::state_tree_node::{PgnStateTreeNode};
use std::fmt::{Display, Formatter};
use crate::utils::Color;
use crate::pgn::tokenize::PgnToken;
use crate::state::{Termination};

use std::fmt::Write;
use crate::pgn::state_tree::PgnStateTree;

pub fn render_tokens(tokens: Vec<PgnToken>) -> String {
    let mut result = String::with_capacity(tokens.len() * 10); // Estimate initial capacity
    let mut indent_level = 0;

    for token in tokens {
        match token {
            PgnToken::StartVariation => {
                indent_level += 1;
                writeln!(result).unwrap();
                write!(result, "{}( ", "    ".repeat(indent_level)).unwrap();
            }
            PgnToken::EndVariation => {
                indent_level -= 1;
                write!(result, ")\n{}", "    ".repeat(indent_level)).unwrap();
            }
            PgnToken::MoveNumberAndPeriods(mn, np) => {
                write!(result, "{}{}", mn, ".".repeat(np)).unwrap();
            }
            PgnToken::Move(m) => write!(result, "{} ", m).unwrap(),
            PgnToken::Tag(tag) => writeln!(result, "{}", tag).unwrap(),
            PgnToken::Comment(c) => write!(result, "{}", c).unwrap(),
            PgnToken::Annotation(a) => write!(result, "{}", a).unwrap(),
            PgnToken::Result(r) => write!(result, "{}", r).unwrap(),
        }
    }

    // Trim trailing whitespace from each line and the end of the string
    result.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

impl Display for PgnStateTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", render_tokens(self.to_tokens()))
    }
}

impl PgnStateTreeNode {
    fn get_san(&self) -> String {
        match self.move_and_san_and_previous_node.clone() {
            None => panic!(),
            Some((_, s, _)) => s
        }
    }
    
    pub(crate) fn to_tokens(&self, render_own_move: bool) -> Vec<PgnToken> {
        let mut res = Vec::new();
        let side_to_move_after_move = self.state_after_move.side_to_move;
        let fullmove_after_move = self.state_after_move.get_fullmove();
        
        if render_own_move {
            // add the current node's move
            let san = self.get_san();
            res.push(PgnToken::Move(san));
        }

        // check for next node
        let optional_next_node = self.next_main_node();
        let next_node = match optional_next_node {
            None => return res, // no next node, return
            Some(ref node) => node.clone() // next node exists, continue
        };
        
        if side_to_move_after_move == Color::White {
            // add next node's fullmove number
            res.push(PgnToken::MoveNumberAndPeriods(fullmove_after_move, 1));
        }
        
        // add next node's move
        let san = next_node.borrow().get_san();
        res.push(PgnToken::Move(san));
        
        // recurse into next variation nodes
        for variation in self.next_variation_nodes() {
            res.push(PgnToken::StartVariation); // add '('
            let num_periods = match side_to_move_after_move {
                Color::White => 1,
                Color::Black => 3
            };
            res.push(PgnToken::MoveNumberAndPeriods(fullmove_after_move, num_periods)); // add fullmove number
            res.append(&mut (*variation).borrow().to_tokens(true)); // recurse into next variation
            res.push(PgnToken::EndVariation); // add ')'
        }
        
        if self.has_variation() && side_to_move_after_move == Color::White {
            // add fullmove number
            res.push(PgnToken::MoveNumberAndPeriods(next_node.borrow().state_after_move.get_fullmove(), 3));
        }

        // recurse into next node
        res.append(&mut next_node.borrow().to_tokens(false));
        
        res
    }
}

impl PgnStateTree {
    pub fn to_tokens(&self) -> Vec<PgnToken> {
        let mut res = Vec::new();
        
        for tag in self.tags.iter() {
            res.push(PgnToken::Tag(format!("[{} \"{}\"]", tag.0, tag.1)));
        }
        
        res.append(&mut (*self.head).borrow().to_tokens(false));
        
        let mut last_node = self.head.clone();
        while let Some(next_node) = last_node.clone().borrow().next_main_node() {
            last_node = next_node;
        };
        let final_state = last_node.borrow().state_after_move.clone();
        match final_state.termination {
            None => (),
            Some(termination) => {
                let result_string = match termination {
                    Termination::Checkmate => {
                        match final_state.side_to_move {
                            Color::White => "0-1",
                            Color::Black => "1-0"
                        }
                    },
                    Termination::Stalemate | Termination::ThreefoldRepetition | Termination::InsufficientMaterial | Termination::FiftyMoveRule => "1/2-1/2",
                };
                res.push(PgnToken::Result(result_string.to_string()));
            }
        }
        
        res
    }
}