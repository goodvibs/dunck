use dunck::engine::conv_net_evaluator::constants::{NUM_OUTPUT_POLICY_MOVES, NUM_TARGET_SQUARE_POSSIBILITIES};
use dunck::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor};
use dunck::engine::conv_net_evaluator::ConvNetEvaluator;
use dunck::engine::evaluation::Evaluation;
use dunck::pgn::PgnStateTree;
use dunck::r#move::MoveFlag;
use dunck::state::{State, Termination};
use dunck::utils::Color;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fs::exists;
use std::str::FromStr;
use rand::Rng;
use tch::nn::OptimizerConfig;
use tch::{nn, Kind, Tensor};
use dunck::engine::conv_net_evaluator::conv_net::ConvNet;

pub const MULTI_PGN_FILE: &str = "data/lichess_elite_db_multi_pgn/accepted.pgn";
pub const MODEL_FILE: &str = "model.safetensors";

pub const NUM_RESIDUAL_BLOCKS: usize = 10;
pub const NUM_FILTERS: i64 = 256;

pub struct LossMetrics {
    pub policy_loss: f64,
    pub value_loss: f64,
    pub total_loss: f64,
}

fn extract_pgns(multi_pgn_file_content: &str) -> Vec<String> {
    let mut pgns = Vec::new();
    let initial_split = multi_pgn_file_content.trim().split("\n\n");
    for split in initial_split {
        let split = split.trim();
        pgns.push(split.to_string());
    }
    pgns
}

fn load_evaluator() -> ConvNetEvaluator {
    let mut evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS);
    if exists(MODEL_FILE).expect("Failed to check if model file exists") {
        println!("Loading model from file...");
        evaluator.model.load(MODEL_FILE).expect("Failed to load model");
    }
    evaluator
}

fn verify_and_save_model(evaluator: &ConvNetEvaluator) {
    println!("Training completed. Saving model...");
    evaluator.model.save(MODEL_FILE).expect("Failed to save model");

    // Verify saved model
    let mut evaluator2 = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS);
    evaluator2.model.load(MODEL_FILE).expect("Failed to load model");
    assert_eq!(evaluator.model.vs.variables().len(), evaluator2.model.vs.variables().len());

    let evaluator2_variables = evaluator2.model.vs.variables();
    for (name, tensor) in evaluator.model.vs.variables() {
        let tensor2 = evaluator2_variables.get(&name).expect("Failed to get tensor");
        assert_eq!(tensor.size(), tensor2.size());
        assert!(Tensor::allclose(&tensor, &tensor2, 1e-6, 1e-6, false));
    }

    println!("Model verified and saved to file");
}

/// Sample a batch of data from a given PGN set
fn get_random_batch_from_pgns(
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

fn get_random_example_from_state_tree(state_tree: PgnStateTree, rng: &mut ThreadRng) -> Option<(State, Evaluation)> {
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
    if num_moves < 75 {
        return None;
    }
    
    let node_idx = rng.gen_range(0..num_moves-1);

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

    let policy: Vec<(dunck::r#move::Move, f64)> = legal_moves
        .into_iter()
        .map(|mv| (mv, if mv == expected_mv { 1.0 } else { 0.0 }))
        .collect();

    Some((initial_state, Evaluation { policy, value }))
}

/// Compute the losses (policy and value) for a given batch of data without updating the model.
fn compute_loss(
    model: &ConvNet,
    data: &[(State, Evaluation)]
) -> LossMetrics {
    let (states, policies, values) = create_batch_tensors(data);

    let (pred_policies, pred_values) = model.forward(&states, false);

    // Apply log_softmax to the policy logits
    let log_probs = pred_policies.log_softmax(-1, Kind::Float);

    // Cross-entropy for policy
    let policy_loss = -(policies * &log_probs)
        .sum_dim_intlist(&[1i64, 2i64, 3i64][..], false, Kind::Float)
        .mean(Kind::Float);

    // MSE for value
    let value_loss = pred_values.mse_loss(&values, tch::Reduction::Mean);

    let total_loss = &policy_loss + &value_loss;

    LossMetrics {
        policy_loss: policy_loss.double_value( & []),
        value_loss: value_loss.double_value( & []),
        total_loss: total_loss.double_value( & [])
    }
}

/// Create batch tensors for states, policies, and values
fn create_batch_tensors(
    training_data: &[(State, Evaluation)]
) -> (Tensor, Tensor, Tensor) {
    let batch_states: Vec<_> = training_data
        .iter()
        .map(|(state, _)| state_to_tensor(state))
        .collect();

    let batch_policies: Vec<_> = training_data
        .iter()
        .map(|(state, eval)| {
            let mut policy = vec![0.0; NUM_OUTPUT_POLICY_MOVES];
            for (mv, prob) in &eval.policy {
                let src_square = mv.get_source().to_perspective_from_white(state.side_to_move);
                let dst_square = mv.get_destination().to_perspective_from_white(state.side_to_move);
                let vetted_promotion = match mv.get_flag() {
                    MoveFlag::Promotion => Some(mv.get_promotion()),
                    _ => None,
                };

                let policy_index = get_policy_index_for_move(src_square, dst_square, vetted_promotion);
                let flat_index = src_square.get_rank() as usize * 8 * NUM_TARGET_SQUARE_POSSIBILITIES as usize
                    + src_square.get_file() as usize * NUM_TARGET_SQUARE_POSSIBILITIES as usize
                    + policy_index as usize;
                policy[flat_index] = *prob;
            }
            Tensor::from_slice(&policy).view([8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64])
        })
        .collect();

    let batch_values: Vec<_> = training_data
        .iter()
        .map(|(_, eval)| Tensor::from_slice(&[eval.value]))
        .collect();

    let states = Tensor::stack(&batch_states, 0).to_kind(Kind::Float);
    let policies = Tensor::stack(&batch_policies, 0).to_kind(Kind::Float);
    let values = Tensor::stack(&batch_values, 0).to_kind(Kind::Float);

    (states, policies, values)
}

/// Update the model parameters given a batch of training data
fn train_batch(
    model: &mut ConvNet,
    optimizer: &mut nn::Optimizer,
    batch_data: &[(State, Evaluation)]
) -> LossMetrics {
    let (states, policies, values) = create_batch_tensors(batch_data);

    let (pred_policies, pred_values) = model.forward(&states, true);

    // Apply log_softmax to the policy logits
    let log_probs = pred_policies.log_softmax(-1, Kind::Float);

    // Cross-entropy for policy: -sum(target * log_pred)
    let policy_loss = -(policies * &log_probs)
        .sum_dim_intlist(&[1i64, 2i64, 3i64][..], false, Kind::Float)
        .mean(Kind::Float);

    // MSE for value
    let value_loss = pred_values.mse_loss(&values, tch::Reduction::Mean);

    let total_loss = &policy_loss + &value_loss;

    optimizer.zero_grad();
    total_loss.backward();
    optimizer.step();

    let policy_loss_scalar = policy_loss.double_value(&[]);
    let value_loss_scalar = value_loss.double_value(&[]);
    let total_loss_scalar = total_loss.double_value(&[]);

    LossMetrics {
        policy_loss: policy_loss_scalar,
        value_loss: value_loss_scalar,
        total_loss: total_loss_scalar,
    }
}

fn main() {
    let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
    let pgns = extract_pgns(&multi_pgn_file_content);

    let mut random_state = rand::thread_rng();
    
    // Training parameters
    let num_iterations = 200;
    let num_batches = 15;
    let num_examples_per_batch = 512;
    let mut learning_rate = 0.0003;

    // Parameters for dynamic LR adjustment
    let patience = 5;
    let reduce_factor = 0.5;
    let mut best_val_loss = f64::INFINITY;
    let mut no_improvement_count = 0;

    let validation_data = get_random_batch_from_pgns(&pgns, num_examples_per_batch, &mut random_state);

    for i in 0..num_iterations {
        println!("|*| Training iteration {}/{} with learning rate {} |*|", i + 1, num_iterations, learning_rate);

        let mut evaluator = load_evaluator();
        let mut optimizer = nn::Adam::default()
            .build(&evaluator.model.vs, learning_rate)
            .expect("Failed to create optimizer");

        for batch_num in 0..num_batches {
            println!("Starting batch {}/{}", batch_num + 1, num_batches);

            // Get fresh training data for this batch
            let training_data = get_random_batch_from_pgns(&pgns, num_examples_per_batch, &mut random_state);

            // Train on the training data
            let train_loss_metrics = train_batch(&mut evaluator.model, &mut optimizer, &training_data);

            // Evaluate on validation data
            let val_loss_metrics = compute_loss(&evaluator.model, &validation_data);

            println!(
                "Batch {}/{} Completed. Training (Policy: {:.4}, Value: {:.4}, Total: {:.4}), Validation (Policy: {:.4}, Value: {:.4}, Total: {:.4})",
                batch_num + 1, num_batches,
                train_loss_metrics.policy_loss, train_loss_metrics.value_loss, train_loss_metrics.total_loss,
                val_loss_metrics.policy_loss, val_loss_metrics.value_loss, val_loss_metrics.total_loss
            );

            // Check if validation improved
            if val_loss_metrics.total_loss < best_val_loss {
                best_val_loss = val_loss_metrics.total_loss;
                no_improvement_count = 0;
            } else {
                no_improvement_count += 1;
                if no_improvement_count >= patience {
                    // Reduce learning rate
                    learning_rate *= reduce_factor;
                    optimizer.set_lr(learning_rate);
                    println!("No validation improvement for {} batches, reducing LR to {}", patience, learning_rate);
                    no_improvement_count = 0;
                    best_val_loss = f64::INFINITY;
                }
            }
        }

        verify_and_save_model(&evaluator);
    }
}

#[cfg(test)]
mod tests {
    use dunck::engine::mcts::{calc_puct_score, MCTS};
    use dunck::r#move::{Move, MoveFlag};
    use dunck::state::State;
    use dunck::utils::{Square};
    use crate::*;

    #[test]
    fn test_training() {
        let expected_move_white = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);
        let expected_move_black = Move::new(Square::F6, Square::G8, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

        let test_states = [
            // white to move
            State::initial(),
            State::from_fen("rnbqkbnr/ppp1pppp/8/3p4/5P2/8/PPPPP1PP/RNBQKBNR w KQkq - 0 2").unwrap(),
            State::from_fen("r1bqkb1r/p2ppppp/1pn2n2/2p5/P7/2P2P1P/1P1PP1P1/RNBQKBNR w KQkq - 0 5").unwrap(),
            State::from_fen("rnbqkbnr/4pppp/8/pppp4/PPP2P2/8/3PP1PP/RNBQKBNR w KQkq - 0 5").unwrap(),
            // black to move
            State::from_fen("rnbqkbnr/ppp1pppp/8/3p4/5P2/5N2/PPPPP1PP/RNBQKB1R b KQkq - 1 2").unwrap(),
            State::from_fen("rnbqkbnr/1pp1pppp/8/p2p4/5P2/5N1P/PPPPP1P1/RNBQKB1R b KQkq - 0 3").unwrap(),
            State::from_fen("rnbqkbnr/2p1pppp/8/1N1p4/1p3P1P/8/P1PPP1P1/R1BQKBNR b KQkq - 0 5").unwrap(),
        ];

        for state in test_states.iter() {
            let legal_moves = state.calc_legal_moves();
            if state.side_to_move == Color::White {
                assert!(legal_moves.contains(&expected_move_white));
            } else {
                assert!(legal_moves.contains(&expected_move_black));
            }
        }

        let mut evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS);
        let mut optimizer = nn::Adam::default().build(&evaluator.model.vs, 0.01).unwrap();

        let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
        let pgns = extract_pgns(&multi_pgn_file_content);
        let rng = &mut rand::thread_rng();

        let mut train_loss_metrics = LossMetrics {
            policy_loss: 0.0,
            value_loss: 0.0,
            total_loss: 0.0,
        };

        for i in 0..10 {
            println!("Starting batch {}/{}", i + 1, 10);
            let random_batch_vec = get_random_batch_from_pgns(&pgns, 50, rng);
            let modified_random_batch_vec = random_batch_vec.iter().map(|(state, _)| {
                let modified_eval = match state.side_to_move {
                    Color::White => Evaluation {
                        policy: vec![(expected_move_white, 1.0)],
                        value: 1.0,
                    },
                Color::Black => Evaluation {
                        policy: vec![(expected_move_black, 1.0)],
                        value: -1.0,
                    },
                };
                (state.clone(), modified_eval)
            }).collect::<Vec<_>>();

            train_loss_metrics = train_batch(&mut evaluator.model, &mut optimizer, &modified_random_batch_vec);

            println!(
                "Batch {}/{} Completed. Training (Policy: {:.4}, Value: {:.4}, Total: {:.4})",
                i + 1, 10,
                train_loss_metrics.policy_loss, train_loss_metrics.value_loss, train_loss_metrics.total_loss,
            );
        }

        assert!(train_loss_metrics.policy_loss < 0.1);
        assert!(train_loss_metrics.value_loss < 0.1);
        assert!(train_loss_metrics.total_loss < 0.1);

        println!();

        for state in test_states {
            let mut mcts = MCTS::new(state.clone(), 2.0, &evaluator, &calc_puct_score, false);
            mcts.run(2);
            if let Some(best_move_node) = mcts.get_best_child_by_visits() {
                let best_move = best_move_node.borrow().mv.clone();
                println!("{}", mcts);
                match state.side_to_move {
                    Color::White => { 
                        assert_eq!(best_move.unwrap(), expected_move_white);
                        assert!(mcts.root.borrow().value > 0.9);
                    },
                    Color::Black => {
                        assert_eq!(best_move.unwrap(), expected_move_black);
                        assert!(mcts.root.borrow().value < -0.9);
                    },
                }
            }
        }
    }
}