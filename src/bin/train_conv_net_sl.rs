use std::fs::{exists};
use std::str::FromStr;
use tch::nn::OptimizerConfig;
use tch::{nn, Device, Kind, Tensor};
use rand::seq::SliceRandom;
use std::time::Instant;
use rand::Rng;
use rand::rngs::ThreadRng;
use dunck::engine::conv_net_evaluator::constants::{NUM_OUTPUT_POLICY_MOVES, NUM_TARGET_SQUARE_POSSIBILITIES};
use dunck::engine::conv_net_evaluator::ConvNetEvaluator;
use dunck::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor};
use dunck::engine::evaluation::Evaluation;
use dunck::pgn::PgnStateTree;
use dunck::state::{State, Termination};
use dunck::r#move::MoveFlag;
use dunck::utils::Color;

pub const MULTI_PGN_FILE: &str = "data/lichess_elite_db_multi_pgn/accepted.pgn";

pub const NUM_RESIDUAL_BLOCKS: usize = 4;
pub const NUM_FILTERS: i64 = 32;

fn extract_pgns(multi_pgn_file_content: &str) -> Vec<String> {
    let mut pgns = Vec::new();
    let initial_split = multi_pgn_file_content.trim().split("\n\n");
    for split in initial_split {
        let split = split.trim();
        pgns.push(split.to_string());
    }
    pgns
}

fn train(pgns: &[String], num_epochs: usize, num_batches_per_epoch: usize, learning_rate: f64) {
    let mut random_state = rand::thread_rng();
    
    let mut evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS, true);

    // Load model if it exists
    if exists("model.pt").expect("Failed to check if model file exists") {
        println!("Loading model from file...");
        evaluator.model.load("model.pt").expect("Failed to load model");
    }

    let mut optimizer = nn::Adam::default()
        .build(&evaluator.model.vs, learning_rate)
        .expect("Failed to create optimizer");
    
    let start_time = Instant::now();
    
    for epoch in 0..num_epochs {
        println!("Starting epoch {}/{}", epoch + 1, num_epochs);

        let mut training_data: Vec<(State, Evaluation)> = Vec::new();
        
        for batch_idx in 0..num_batches_per_epoch {
            // println!("Starting batch {}/{}", batch_idx + 1, num_batches);
            
            let mut pgn;
            
            loop {
                pgn = match pgns.choose(&mut random_state) {
                    Some(pgn) => pgn,
                    None => { 
                        println!("Failed to choose PGN. Retrying...");
                        continue;
                    }
                };
                
                let state_tree = match PgnStateTree::from_str(pgn.as_str()) {
                    Ok(state_tree) => state_tree,
                    Err(_) => {
                        println!("Failed to parse PGN. Retrying...");
                        continue;
                    }
                };
                
                let training_example = match get_random_example_from_state_tree(state_tree, &mut random_state) {
                    Some(example) => example,
                    None => {
                        println!("Failed to get random example from state tree. Retrying...");
                        continue;
                    }
                };
                
                training_data.push(training_example);
                break;
            }
        }
        
        train_epoch(&mut evaluator, &mut optimizer, &training_data);
    }
    
    println!("Training completed. Total time elapsed: {:.2}s", start_time.elapsed().as_secs_f32());
    evaluator.model.save("model.pt").expect("Failed to save model");
    println!("Model saved to file");
}


fn get_random_example_from_state_tree(state_tree: PgnStateTree, rng: &mut ThreadRng) -> Option<(State, Evaluation)> {
    let mut num_moves = 0;
    
    let mut current_node = state_tree.head.clone();
    while let Some(next_node) = current_node.clone().borrow().next_main_node() {
        current_node = next_node;
        num_moves += 1;
    };
    
    let winner = match current_node.borrow().state_after_move.termination {
        Some(Termination::Checkmate) => {
            if current_node.borrow().state_after_move.side_to_move == Color::White {
                Some(Color::Black)
            } else {
                Some(Color::White)
            }
        },
        Some(_) => None,
        None => return None
    };
    
    if num_moves < 10 {
        None
    } else {
        let mut current_node = state_tree.head.clone();
        let mut next_node = current_node.clone().borrow().next_main_node().unwrap();
        
        let node_idx = rng.gen_range(0..num_moves - 1);
        for _ in 0..node_idx {
            let next_next_node = next_node.borrow().next_main_node().unwrap();
            current_node = next_node;
            next_node = next_next_node;
        }

        let initial_state = current_node.borrow().state_after_move.clone();
        let legal_moves = initial_state.calc_legal_moves();
        
        let expected_mv = next_node.borrow().move_and_san_and_previous_node.clone().unwrap().0.clone();

        if !legal_moves.iter().any(|mv| *mv == expected_mv) {
            initial_state.board.print();
            println!("Expected move: {:?}", expected_mv);
        }
        
        assert!(legal_moves.iter().any(|mv| *mv == expected_mv));

        let value = match winner {
            Some(winner) => {
                if winner == initial_state.side_to_move {
                    1.0
                } else {
                    -1.0
                }
            },
            None => 0.0
        };
        
        let mut policy = Vec::new();
        
        for legal_mv in legal_moves {
            policy.push((legal_mv, if legal_mv == expected_mv { 1.0 } else { 0.0 }));
        }
        
        let evaluation = Evaluation {
            policy,
            value
        };
        
        Some((initial_state, evaluation))
    }
}


fn train_epoch(
    evaluator: &mut ConvNetEvaluator,
    optimizer: &mut nn::Optimizer,
    training_data: &[(State, Evaluation)],
) {
    let mut indices: Vec<usize> = (0..training_data.len()).collect();
    indices.shuffle(&mut rand::thread_rng());

    for chunk in indices.chunks(training_data.len()) {
        // Prepare batch tensors
        let batch_states: Vec<_> = chunk
            .iter()
            .map(|&i| state_to_tensor(&training_data[i].0))
            .collect();

        // Convert policy vectors to tensors
        let batch_policies: Vec<_> = chunk
            .iter()
            .map(|&i| {
                let mut policy = vec![0.0; NUM_OUTPUT_POLICY_MOVES];
                let state = &training_data[i].0;
                for (mv, prob) in &training_data[i].1.policy {
                    let src_square_from_current_perspective = mv.get_source().to_perspective_from_white(state.side_to_move);
                    let dst_square_from_current_perspective = mv.get_destination().to_perspective_from_white(state.side_to_move);
                    let vetted_promotion = match mv.get_flag() {
                        MoveFlag::Promotion => Some(mv.get_promotion()),
                        _ => None
                    };

                    let policy_index = get_policy_index_for_move(
                        src_square_from_current_perspective,
                        dst_square_from_current_perspective,
                        vetted_promotion
                    );

                    policy[policy_index as usize] = *prob;
                }
                Tensor::from_slice(&policy).view([8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64])
            })
            .collect();

        // Convert values to tensors
        let batch_values: Vec<_> = chunk
            .iter()
            .map(|&i| Tensor::from_slice(&[training_data[i].1.value]))
            .collect();

        // Stack into batch tensors
        let states = Tensor::stack(&batch_states, 0).to_kind(Kind::Float);
        let policies = Tensor::stack(&batch_policies, 0).to_kind(Kind::Float);
        let values = Tensor::stack(&batch_values, 0).to_kind(Kind::Float);

        // Forward pass
        let (pred_policies, pred_values) = evaluator.model.forward(&states, true);

        // Calculate losses
        let policy_loss = pred_policies.kl_div(&policies, tch::Reduction::Mean, false);
        let policy_loss_scalar = policy_loss.double_value(&[]);
        let value_loss = pred_values.mse_loss(&values, tch::Reduction::Mean);
        let value_loss_scalar = value_loss.double_value(&[]);
        let total_loss = policy_loss + value_loss;
        let total_loss_scalar = total_loss.double_value(&[]);

        // Backward pass and optimization
        optimizer.zero_grad();
        total_loss.backward();
        optimizer.step();

        // Log training progress
        println!(
            "Loss - Policy: {:.4}, Value: {:.4}, Total: {:.4}",
            policy_loss_scalar, value_loss_scalar, total_loss_scalar
        );
    }
}

fn main() {
    let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
    let pgns = extract_pgns(&multi_pgn_file_content);
    for i in 0..200 {
        let learning_rate;
        if i < 5 {
            learning_rate = 0.05;
        } else if i < 20 {
            learning_rate = 0.01;
        } else if i < 50 {
            learning_rate = 0.005;
        } else {
            learning_rate = 0.001;
        }
        
        println!("|*| Training iteration {}/200 with learning rate {} |*|", i + 1, learning_rate);
        
        train(&pgns, 1, 500, learning_rate);
    }
}