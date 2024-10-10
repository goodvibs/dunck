use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::{Chars, FromStr};
use indexmap::IndexMap;
use crate::pgn::error::PgnParseError;
use crate::pgn::pgn_move_tree_node::PgnMoveTreeNode;
use crate::pgn::PgnMoveTree;
use crate::pgn::tokenize::{tokenize_pgn, PgnToken};
use crate::state::State;

fn validate_tag_placement(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut can_tag_be_placed = true;
    
    for token in tokens {
        match token {
            PgnToken::Tag(tag) => {
                if !can_tag_be_placed {
                    return Err(PgnParseError::InvalidTagPlacement(tag.clone()));
                }
            }
            _ => {
                can_tag_be_placed = false;
            }
        }
    }
    
    Ok(())
}

fn validate_result_placement(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut results_placed = false;
    
    for token in tokens {
        match token {
            PgnToken::Result(result) => {
                if results_placed {
                    return Err(PgnParseError::InvalidResultPlacement(result.clone()));
                }
                results_placed = true;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// Ensure that all variations start after a move
fn validate_variation_start_placement(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut last_token_was_move = false;
    
    for token in tokens {
        match token {
            PgnToken::Move(_) => {
                last_token_was_move = true;
            }
            PgnToken::StartVariation => {
                if !last_token_was_move {
                    return Err(PgnParseError::InvalidVariationStart("Variation does not start after a move".to_string()));
                }
                last_token_was_move = false;
            }
            PgnToken::MoveNumber(_) | PgnToken::Tag(_) | PgnToken::Result(_) => {
                last_token_was_move = false;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// Ensure that all variations end after a move
fn validate_variation_end_placement(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut last_token_was_move = false;
    
    for token in tokens {
        match token {
            PgnToken::Move(_) => {
                last_token_was_move = true;
            }
            PgnToken::EndVariation => {
                if !last_token_was_move {
                    return Err(PgnParseError::InvalidVariationClosure("Variation does not end after a move".to_string()));
                }
            }
            PgnToken::StartVariation | PgnToken::MoveNumber(_) | PgnToken::Tag(_) | PgnToken::Result(_) => {
                last_token_was_move = false;
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn validate_variation_closure(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut open_variations = 0;
    
    for token in tokens {
        match token {
            PgnToken::StartVariation => {
                open_variations += 1;
            }
            PgnToken::EndVariation => {
                open_variations -= 1;
            }
            _ => {}
        }
    }
    
    if open_variations != 0 {
        return Err(PgnParseError::InvalidVariationClosure("Variation is not closed".to_string()));
    }
    
    Ok(())
}

fn validate_move_numbers(tokens: &Vec<PgnToken>) -> Result<(), PgnParseError> {
    let mut stack = Vec::new();
    let mut halfmove = 1;
    
    for token in tokens {
        match token {
            PgnToken::MoveNumber(found_fullmove) => {
                let expected_fullmove = (halfmove + 1) / 2;
                if found_fullmove != &expected_fullmove {
                    return Err(PgnParseError::IncorrectMoveNumber(found_fullmove.to_string()));
                }
            }
            PgnToken::Move(_) => {
                halfmove += 1;
            }
            PgnToken::StartVariation => {
                stack.push(halfmove);
                halfmove -= 1;
            }
            PgnToken::EndVariation => {
                halfmove = match stack.pop() {
                    Some(halfmove) => halfmove,
                    None => return Err(PgnParseError::InvalidVariationClosure("Variation is not closed".to_string()))
                };
            }
            _ => {}
        }
    }
    
    Ok(())
}

impl PgnMoveTree {
    fn from_tokens(tokens: Vec<PgnToken>) -> Result<PgnMoveTree, PgnParseError> {
        validate_tag_placement(&tokens)?;
        validate_result_placement(&tokens)?;
        validate_variation_start_placement(&tokens)?;
        validate_variation_end_placement(&tokens)?;
        validate_variation_closure(&tokens)?;
        validate_move_numbers(&tokens)?;

        let mut pgn_move_tree = PgnMoveTree::new();

        let mut current_node = pgn_move_tree.head;
        let mut node_stack = Vec::new();
        
        let mut tokens = tokens.iter().peekable();
        
        while let Some(token) = tokens.next() {
            match token {
                PgnToken::Tag(tag) => {
                    // let (key, value) = parse_tag(tag)?;
                    // pgn_move_tree.tags.insert(key, value);
                }
                PgnToken::MoveNumber(move_number) => {
                    // todo!()
                }
                PgnToken::Move(mv) => unsafe {
                    let initial_state = (*current_node).state_after_move.clone();
                    let legal_moves = initial_state.get_legal_moves();
                    
                    let mut found_match = false;
                    
                    for legal_move in &legal_moves {
                        let mut new_state = initial_state.clone();
                        new_state.make_move(*legal_move);
                        
                        if legal_move.san(&initial_state, &new_state, &legal_moves) == *mv {
                            found_match = true;
                            
                            current_node = PgnMoveTreeNode::new_raw_linked_to_previous(*legal_move, mv.to_string(), current_node, new_state);
                            
                            break;
                        }
                    }
                    
                    if !found_match {
                        return Err(PgnParseError::IllegalMove(mv.to_string()));
                    }
                }
                PgnToken::StartVariation => unsafe {
                    node_stack.push(current_node);
                    current_node = match (*current_node).move_and_san_and_previous_node {
                        Some((_, _, previous_node)) => previous_node,
                        None => return Err(PgnParseError::InvalidVariationStart("Variation does not start after a move".to_string()))
                    }
                }
                PgnToken::EndVariation => {
                    current_node = match node_stack.pop() {
                        Some(node) => node,
                        None => return Err(PgnParseError::InvalidVariationClosure("There is no open variation".to_string()))
                    }
                }
                PgnToken::Comment(_) => {
                    // todo!()
                }
                PgnToken::Annotation(_) => {
                    // todo!()
                }
                PgnToken::Result(result) => {
                    match result.as_str() {
                        "1-0" => {
                            // todo!()
                        }
                        "0-1" => {
                            // todo!()
                        }
                        "1/2-1/2" => {
                            // todo!()
                        }
                        _ => {
                            return Err(PgnParseError::InvalidResult(result.to_string()));
                        }
                    }
                }
            }
        }
        
        Ok(pgn_move_tree)
    }
}

impl FromStr for PgnMoveTree {
    type Err = PgnParseError;

    fn from_str(pgn: &str) -> Result<PgnMoveTree, PgnParseError> {
        let tokens = tokenize_pgn(pgn)?;
        PgnMoveTree::from_tokens(tokens)
    }
}