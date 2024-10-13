use crate::pgn::pgn_move_tree_node::{PgnMoveTreeNode, PgnMoveTreeNodePtr};
use crate::pgn::{PgnMoveTree};
use indexmap::IndexMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::str::FromStr;
use crate::miscellaneous::Color;
use crate::pgn::tokenize::PgnToken;
use crate::state::State;

pub fn render_tokens(tokens: Vec<PgnToken>) -> String {
    let mut res = String::new();
    for token in tokens {
        res += &*format!("{}", token);
    }
    res
}

impl Display for PgnToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnToken::Tag(tag) => write!(f, "{}\n", tag),
            PgnToken::Move(m) => write!(f, "{} ", m),
            PgnToken::MoveNumber(n) => write!(f, "{}. ", n),
            PgnToken::StartVariation => write!(f, "\n( "),
            PgnToken::EndVariation => write!(f, ")\n"),
            PgnToken::Comment(c) => write!(f, "{}", c),
            PgnToken::Annotation(a) => write!(f, "{}", a),
            PgnToken::Result(r) => write!(f, " {}", r),
        }
    }
}

impl Display for PgnMoveTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", render_tokens(self.to_tokens()))
    }
}

impl PgnMoveTreeNode {
    // pub(crate) fn pgn(&self, should_render_variations: bool, mut should_remind_fullmove: bool, prepend_tabs: u8) -> String {
    //     let mut res = String::new();
    //     let (_, san, _): (Option<u8>, String, Option<u8>) = match self.move_and_san_and_previous_node.clone() {
    //         None => (None, "".to_string(), None),
    //         Some((_, s, _)) => (None, s, None)
    //     };
    //     if self.state_after_move.halfmove != 0 {
    //         res += match self.state_after_move.side_to_move {
    //             Color::White => match should_remind_fullmove {
    //                 true => format!("{}...{}", self.state_after_move.get_fullmove() - 1, san),
    //                 false => format!(" {}", san),
    //             },
    //             Color::Black => format!("{}{}.{}", if self.state_after_move.halfmove > 1 && !should_remind_fullmove { " " } else { "" }, self.state_after_move.get_fullmove(), san)
    //         }.as_str();
    //     }
    //     should_remind_fullmove = false;
    //     if should_render_variations && self.has_variation() {
    //         let variations = self.next_variation_nodes();
    //         res += format!("\n{}( ", "    ".repeat(prepend_tabs as usize + 1)).as_str();
    //         for variation in variations {
    //             res += &*format!("{}", (*variation).borrow().pgn(true, true, prepend_tabs + 1));
    //         }
    //         res += format!(" )\n{}", "    ".repeat(prepend_tabs as usize)).as_str();
    //         should_remind_fullmove = true;
    //     }
    //     if let Some(next_node) = self.next_main_node() {
    //         res += &*format!("{}", (*next_node).borrow().pgn(should_render_variations, should_remind_fullmove, prepend_tabs));
    //     }
    //     res
    // }
    
    pub(crate) fn to_tokens(&self, render_self: bool) -> Vec<PgnToken> {
        let mut res = Vec::new();
        
        if render_self {
            let san = match self.move_and_san_and_previous_node.clone() {
                None => panic!(),
                Some((_, s, _)) => s
            };
            res.push(PgnToken::Move(san));
        }

        let optional_next_node = self.next_main_node();
        let next_node = match optional_next_node {
            None => return res,
            Some(ref node) => {
                node.clone()
            }
        };
        let san = match next_node.borrow().move_and_san_and_previous_node.clone() {
            None => panic!(),
            Some((_, s, _)) => s
        };

        res.push(PgnToken::Move(san));

        if self.has_variation() {
            res.push(PgnToken::StartVariation);
            let variations = self.next_variation_nodes();
            for variation in variations {
                res.append(&mut (*variation).borrow().to_tokens(true));
            }
            res.push(PgnToken::EndVariation);
        }

        res.append(&mut next_node.borrow().to_tokens(false));
        
        res
    }
}

impl PgnMoveTree {
    // fn tags_pgn(&self) -> String {
    //     let mut res = String::new();
    //     for tag in self.tags.iter() {
    //         res += &*format!("[{} \"{}\"]\n", tag.0, tag.1);
    //     }
    //     res
    // }

    // fn pgn_helper(&self, should_render_variations: bool) -> String {
    //     let mut res = String::new();
    //     res += &*format!("{}", (*self.head).borrow().pgn(should_render_variations, true, 0));
    //     res
    // }
    // 
    // pub fn pgn(&self) -> String {
    //     if self.tags.is_empty() {
    //         return self.pgn_helper(true);
    //     }
    //     format!("{}\n{}", self.tags_pgn(), self.pgn_helper(true))
    // }
    // 
    // pub fn main_line_pgn(&self) -> String {
    //     self.pgn_helper(false)
    // }
    
    // pub fn traverser(&self) -> PgnMoveTreeTraverser {
    //     PgnMoveTreeTraverser::new(self)
    // }
    
    pub fn to_tokens(&self) -> Vec<PgnToken> {
        let mut res = Vec::new();
        
        for tag in self.tags.iter() {
            res.push(PgnToken::Tag(format!("[{} \"{}\"]", tag.0, tag.1)));
        }
        
        res.append(&mut (*self.head).borrow().to_tokens(false));
        
        res
    }
}

// impl Display for PgnMoveTree {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.pgn())
//     }
// }

// impl Debug for PgnMoveTree {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.tags_pgn())
//     }
// }