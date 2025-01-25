use std::str::FromStr;
use rand::prelude::{SliceRandom, ThreadRng};
use rand::Rng;
use tch::{Kind, Tensor};
use crate::engine::evaluation::Evaluation;
use crate::pgn::PgnStateTree;
use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;

pub fn print_tensor_stats(tensor: &Tensor, message: &str) {
    println!("{}", message);
    println!("-- sum: {}", tensor.sum(Kind::Float).double_value(&[]));
    println!("-- mean: {}", tensor.mean(Kind::Float).double_value(&[]));
    println!("-- std: {}", tensor.std(true).double_value(&[]));
    println!("-- max: {}", tensor.max().double_value(&[]));
    println!("-- min: {}", tensor.min().double_value(&[]));
}

pub fn extract_pgns(multi_pgn_file_content: &str) -> Vec<String> {
    let mut pgns = Vec::new();
    let initial_split = multi_pgn_file_content.trim().split("\n\n");
    for split in initial_split {
        let split = split.trim();
        pgns.push(split.to_string());
    }
    pgns
}

/// Sample a batch of data from a given PGN set
pub fn get_labeled_random_batch_from_pgns(
    pgns: &[String],
    num_samples: usize,
    random_state: &mut ThreadRng
) -> Vec<(State, Evaluation)> {
    let mut data = Vec::with_capacity(num_samples);
    for _ in 0..num_samples {
        let mut pgn;
        loop {
            pgn = match pgns.choose(random_state) {
                Some(pgn) => pgn,
                None => continue,
            };

            let state_tree = match PgnStateTree::from_str(pgn.as_str()) {
                Ok(state_tree) => state_tree,
                Err(_) => continue,
            };

            let example = match get_random_example_from_state_tree(state_tree, random_state) {
                Some(example) => example,
                None => continue,
            };

            data.push(example);
            break;
        }
    }
    data
}

pub fn get_random_example_from_state_tree(state_tree: PgnStateTree, rng: &mut ThreadRng) -> Option<(State, Evaluation)> {
    let mut nodes = Vec::new();
    let mut num_moves = 0;

    let mut current_node = state_tree.head.clone();
    while let Some(next_node) = current_node.clone().borrow().next_main_node() {
        nodes.push(current_node.clone());
        current_node = next_node;
        num_moves += 1;
    }

    // Determine the winner from the final state
    let winner = match current_node.borrow().state_after_move.termination {
        Some(Termination::Checkmate) => {
            if current_node.borrow().state_after_move.side_to_move == Color::White {
                Some(Color::Black)
            } else {
                Some(Color::White)
            }
        },
        Some(_) => None,
        None => return None,
    };

    // Ensure sufficient moves
    if num_moves < 40 {
        return None;
    }

    let node_idx = rng.gen_range(30..num_moves-1);

    let selected_node = nodes[node_idx].clone();
    let next_node = selected_node.borrow().next_main_node().unwrap();

    let initial_state = selected_node.borrow().state_after_move.clone();
    let legal_moves = initial_state.calc_legal_moves();
    let expected_mv = next_node.borrow().move_and_san_and_previous_node.as_ref().unwrap().0.clone();

    assert!(legal_moves.iter().any(|mv| *mv == expected_mv));

    let value = match winner {
        Some(winner) => {
            if winner == initial_state.side_to_move { 1.0 } else { -1.0 }
        },
        None => 0.0,
    };

    let policy: Vec<(Move, f64)> = legal_moves
        .into_iter()
        .map(|mv| (mv, if mv == expected_mv { 1.0 } else { 0.0 }))
        .collect();
    
    // println!("FEN: {}", initial_state.to_fen());
    // initial_state.board.print();
    // println!("Expected move: {}", expected_mv);
    // println!("Winner: {:?}", winner);
    // println!("Value: {}", value);

    Some((initial_state, Evaluation { policy, value }))
}