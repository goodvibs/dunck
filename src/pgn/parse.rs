use crate::pgn::error::PgnParseError;
use crate::pgn::state_tree::PgnStateTree;
use crate::pgn::state_tree_node::PgnStateTreeNode;
use crate::pgn::tokenize::{PgnToken};
use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;

fn validate_tag_placement(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
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

fn validate_result_placement(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
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
fn validate_variation_start_placement(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
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
            PgnToken::MoveNumberAndPeriods(_, _) | PgnToken::Tag(_) | PgnToken::Result(_) => {
                last_token_was_move = false;
            }
            _ => {}
        }
    }
    
    Ok(())
}

/// Ensure that all variations end after a move
fn validate_variation_end_placement(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
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
            PgnToken::StartVariation | PgnToken::MoveNumberAndPeriods(_, _) | PgnToken::Tag(_) | PgnToken::Result(_) => {
                last_token_was_move = false;
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn validate_variation_closure(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
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

fn validate_move_numbers(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
    let mut stack = Vec::new();
    let mut halfmove = 1;
    
    for token in tokens {
        match token {
            PgnToken::MoveNumberAndPeriods(found_fullmove, _) => {
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

fn validate(tokens: &[PgnToken]) -> Result<(), PgnParseError> {
    validate_tag_placement(tokens)?;
    validate_result_placement(tokens)?;
    validate_variation_start_placement(tokens)?;
    validate_variation_end_placement(tokens)?;
    validate_variation_closure(tokens)?;
    validate_move_numbers(tokens)?;
    
    Ok(())
}

fn find_san_match(initial_state: &State, legal_moves: &[Move], expected_san: &str) -> Option<(Move, String, State)> {
    let update_termination = expected_san.ends_with("#");
    
    for legal_move in legal_moves {
        let mut new_state = initial_state.clone();
        new_state.make_move(*legal_move);
        if update_termination {
            new_state.check_and_update_termination();
        }
        
        let san = legal_move.to_san(&initial_state, &new_state, legal_moves);
        if san == expected_san {
            return Some((*legal_move, san, new_state));
        }
    }
    
    None
}

impl PgnStateTree {
    pub fn from_tokens(tokens: &[PgnToken]) -> Result<PgnStateTree, PgnParseError> {
        validate(tokens)?;

        let pgn_move_tree = PgnStateTree::new();

        let mut current_node = pgn_move_tree.head.clone();
        let mut node_stack = Vec::new();
        
        let mut tokens = tokens.iter().peekable();
        
        while let Some(token) = tokens.next() {
            match token {
                PgnToken::Tag(tag) => {
                    // let (key, value) = parse_tag(tag)?;
                    // pgn_move_tree.tags.insert(key, value);
                }
                PgnToken::MoveNumberAndPeriods(move_number, num_periods) => {
                    // todo!()
                }
                PgnToken::Move(mv) => {
                    let initial_state = (*current_node).borrow().state_after_move.clone();
                    let legal_moves = initial_state.calc_legal_moves();
                    
                    match find_san_match(&initial_state, &legal_moves, mv) {
                        Some((found_move, _, new_state)) => {
                            current_node = PgnStateTreeNode::new_linked_to_previous(found_move, mv.to_string(), current_node, new_state);
                        }
                        None => return Err(PgnParseError::IllegalMove(mv.to_string()))
                    }
                }
                PgnToken::StartVariation => {
                    node_stack.push(current_node.clone());
                    let move_and_san_and_previous_node = &current_node.borrow().move_and_san_and_previous_node.clone();
                    current_node = match move_and_san_and_previous_node {
                        Some((_, _, previous_node)) => previous_node.clone(), // Clone the Rc to get a new reference
                        None => return Err(PgnParseError::InvalidVariationStart("Variation does not start after a move".to_string())),
                    };
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
                        "1-0" => { // Todo: Add support for time-related game results
                            let mut node = current_node.borrow_mut();
                            if node.state_after_move.side_to_move == Color::Black {
                                node.state_after_move.termination = Some(Termination::Checkmate);
                            }
                            // node.state_after_move.termination = Some(Termination::Checkmate);
                            // assert_eq!(node.state_after_move.side_to_move, Color::Black);
                        }
                        "0-1" => {
                            let mut node = current_node.borrow_mut();
                            if node.state_after_move.side_to_move == Color::White {
                                node.state_after_move.termination = Some(Termination::Checkmate);
                            }
                            // node.state_after_move.termination = Some(Termination::Checkmate);
                            // assert_eq!(node.state_after_move.side_to_move, Color::White);
                        }
                        "1/2-1/2" => {
                            let mut node = current_node.borrow_mut();
                            node.state_after_move.termination = Some(Termination::Stalemate);
                        }
                        "*" => {
                            // Todo: add support
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