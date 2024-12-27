use dunck::engine::conv_net_evaluator::constants::{NUM_POSITION_BITS, NUM_TARGET_SQUARE_POSSIBILITIES};
use dunck::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor, DEVICE};
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
use dunck::engine::conv_net_evaluator::combined_policy_value_network::CombinedPolicyValueNetwork;
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

/// Helper function to calculate losses and optionally update the model
fn run_model(
    model: &dyn CombinedPolicyValueNetwork,
    optimizer: Option<&mut nn::Optimizer>,
    batch_data: &[(State, Evaluation)],
) -> LossMetrics {
    let num_examples = batch_data.len();
    assert!(num_examples > 0);
    
    let is_training = optimizer.is_some();
    
    let (input_states, expected_policies, expected_values) = create_batch_tensors(batch_data);
    
    assert_eq!(input_states.size(), [num_examples as i64, NUM_POSITION_BITS as i64, 8, 8]);
    assert_eq!(expected_policies.size(), [num_examples as i64, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
    assert_eq!(expected_values.size(), [num_examples as i64, 1]);

    // Forward pass
    let (predicted_policies, predicted_values) = model.forward(&input_states, is_training);
    
    assert_eq!(predicted_policies.size(), expected_policies.size());
    assert_eq!(predicted_values.size(), expected_values.size());
    
    // let policy_loss = predicted_policies.mse_loss(&expected_policies, tch::Reduction::Mean);

    let policy_loss = predicted_policies.cross_entropy_loss::<Tensor>(&expected_policies, None, tch::Reduction::Mean, -100, 0.) * 10000.;
    
    // let policy_loss = predicted_policies.kl_div(&expected_policies, tch::Reduction::Mean, false);
    
    assert_eq!(policy_loss.size(), [] as [i64; 0]);

    // MSE for value
    let value_loss = predicted_values.mse_loss(&expected_values, tch::Reduction::Mean);
    
    assert_eq!(value_loss.size(), [] as [i64; 0]);

    // Total loss
    let total_loss = &policy_loss + &value_loss;
    
    assert_eq!(total_loss.size(), [] as [i64; 0]);

    // Update model if optimizer is provided
    if let Some(opt) = optimizer {
        opt.zero_grad();
        total_loss.backward();
        opt.step();
    }

    // Return losses as scalars
    LossMetrics {
        policy_loss: policy_loss.double_value(&[]),
        value_loss: value_loss.double_value(&[]),
        total_loss: total_loss.double_value(&[]),
    }
}

/// Compute the losses (policy and value) for a given batch of data without updating the model
fn compute_loss(
    model: &dyn CombinedPolicyValueNetwork,
    batch_data: &[(State, Evaluation)],
) -> LossMetrics {
    run_model(model, None, batch_data)
}

/// Update the model parameters given a batch of training data
fn train_batch(
    model: &ConvNet,
    optimizer: &mut nn::Optimizer,
    batch_data: &[(State, Evaluation)],
) -> LossMetrics {
    run_model(model, Some(optimizer), batch_data)
}

/// Create batch tensors for states, policies, and values
fn create_batch_tensors(training_data: &[(State, Evaluation)]) -> (Tensor, Tensor, Tensor) {
    let mut batch_states = Vec::new();
    let mut batch_policies = Vec::new();
    let mut batch_values = Vec::new();

    for (state, eval) in training_data {
        // Process the state tensor
        batch_states.push(state_to_tensor(state));

        // Create a blank policy tensor and fill it
        let policy_tensor = Tensor::zeros(
            [8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64],
            (Kind::Float, *DEVICE),
        );
        for (mv, prob) in &eval.policy {
            let src_square_from_current_perspective = mv.get_source().to_perspective_from_white(state.side_to_move);
            let dst_square_from_current_perspective = mv.get_destination().to_perspective_from_white(state.side_to_move);
            let vetted_promotion = match mv.get_flag() {
                MoveFlag::Promotion => Some(mv.get_promotion()),
                _ => None,
            };

            let policy_index = get_policy_index_for_move(src_square_from_current_perspective, dst_square_from_current_perspective, vetted_promotion);
            assert!(
                policy_index < NUM_TARGET_SQUARE_POSSIBILITIES,
                "Invalid policy index"
            );

            // Fill the tensor directly using indexing
            let _ = policy_tensor
                .get(src_square_from_current_perspective.get_rank() as i64)
                .get(src_square_from_current_perspective.get_file() as i64)
                .get(policy_index as i64)
                .fill_(*prob);
        }
        batch_policies.push(policy_tensor);

        // Add the value tensor
        batch_values.push(Tensor::from_slice(&[eval.value]));
    }

    // Stack tensors for batching
    let states = Tensor::stack(&batch_states, 0).to_kind(Kind::Float);
    let policies = Tensor::stack(&batch_policies, 0).to_kind(Kind::Float);
    let values = Tensor::stack(&batch_values, 0).to_kind(Kind::Float);

    println!(
        "Batch created: states: {:?}, policies: {:?}, values: {:?}",
        states.size(),
        policies.size(),
        values.size()
    );

    (states, policies, values)
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
    let patience = 3;
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
    use dunck::engine::evaluation::Evaluator;
    use std::iter::zip;
    use dunck::engine::conv_net_evaluator::constants::{NUM_BITS_PER_BOARD, NUM_POSITION_BITS};
    use dunck::r#move::{Move, MoveFlag};
    use dunck::state::State;
    use dunck::utils::{Square};
    use crate::*;

    struct DummyNet;

    impl CombinedPolicyValueNetwork for DummyNet {
        fn forward(&self, x: &Tensor, train: bool) -> (Tensor, Tensor) {
            assert_eq!(x.size().len(), 4);
            assert_eq!(x.size()[1..4], [NUM_POSITION_BITS as i64, 8, 8]);
            let batch_size = x.size()[0];
            assert!(batch_size > 0);

            // Create policy tensor initialized to very small values
            let mut policy = Tensor::zeros(&[batch_size, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
            let _ = policy.fill_(0.);  // Fill with very small values

            let mut value = Tensor::zeros(&[batch_size, 1], (Kind::Float, *DEVICE));

            // Get the side to move from the input tensor (channel 12)
            let side_to_move_channel = x.narrow(1, NUM_BITS_PER_BOARD as i64, 1);  // Get the side-to-move channel
            
            for i in 0..batch_size {
                let is_white = side_to_move_channel.get(i).mean(Kind::Float).double_value(&[]) > 0.5;

                if is_white {
                    // For white: e2e4 (index 1)
                    let _ = policy.get(i)
                        .get(1)  // e2 rank
                        .get(4)  // e2 file
                        .get(1)  // move index for queen-like move up 2 squares
                        .fill_(1000.0);

                    let _ = value.get(i).fill_(1.0);
                } else {
                    // For black: g8f6 (index 65)
                    let _ = policy.get(i)
                        .get(0)  // g8 rank from black's perspective
                        .get(1)  // g8 file from black's perspective
                        .get(65) // move index for knight 2 up, 1 right
                        .fill_(1000.0);

                    let _ = value.get(i).fill_(-1.0);
                }
            }

            assert_eq!(policy.size().len(), value.size().len() + 2);

            (policy, value)
        }
    }
    
    struct DummyEvaluator {
        model: DummyNet
    }
    
    impl DummyEvaluator {
        fn new() -> Self {
            Self {
                model: DummyNet
            }
        }
        
        fn evaluate(&self, state: &State) -> Evaluation {
            let state_tensor = state_to_tensor(state);
            let input_tensor = Tensor::stack(&[state_tensor], 0);
            let (policy_logits, value) = self.model.forward(&input_tensor, false);

            let policy = policy_logits.softmax(-1, Kind::Float);

            let legal_moves = state.calc_legal_moves();
            let mut priors = Vec::with_capacity(legal_moves.len());
            let mut sum = 0.;

            for mv in &legal_moves {
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

                let prior = policy.double_value(&[
                    0,
                    src_square_from_current_perspective.get_rank() as i64,
                    src_square_from_current_perspective.get_file() as i64,
                    policy_index as i64
                ]).max(0.);

                priors.push(prior);
                sum += prior;
            }

            let policy = zip(legal_moves, priors)
                .map(|(mv, prior)| (mv, prior / sum))
                .collect();

            Evaluation {
                policy,
                value: value.double_value(&[]),
            }
        }
    }

    #[test]
    fn test_dummy_net_training() {
        let expected_move_white = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);
        let expected_move_black = Move::new(Square::F6, Square::G8, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

        let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
        let pgns = extract_pgns(&multi_pgn_file_content);
        let rng = &mut rand::thread_rng();
        
        let model = DummyNet;

        println!("Computing loss for {} samples", 500);
        let random_batch_vec = get_random_batch_from_pgns(&pgns, 500, rng);
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

        let loss_metrics = compute_loss(&model, &modified_random_batch_vec);

        println!(
            "Batch loss computed. Policy loss: {:.4}, Value loss: {:.4}, Total loss: {:.4}",
            loss_metrics.policy_loss, loss_metrics.value_loss, loss_metrics.total_loss,
        );

        assert!(loss_metrics.policy_loss < 0.001);
        assert_eq!(loss_metrics.value_loss, 0.);
        assert!(loss_metrics.total_loss < 0.001);
    }
    
    #[test]
    fn test_dummy_net_inference() {
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

        let evaluator = DummyEvaluator::new();

        for state in test_states.iter() {
            let is_white = state.side_to_move == Color::White;
            let expected_move = if is_white { expected_move_white } else { expected_move_black };

            let legal_moves = state.calc_legal_moves();
            assert!(legal_moves.contains(&expected_move));
            
            let expected_value = if is_white { 1.0 } else { -1.0 };
            let evaluation = evaluator.evaluate(state);

            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert!(prob > 0.7, "Expected move: {:?}, Prob: {}", mv, prob);
                } else {
                    assert!(prob < 0.05, "Unexpected move: {:?}, Prob: {}", mv, prob);
                }
            }

            assert_eq!(evaluation.value, expected_value);
        }
    }

    #[test]
    fn test_training_conv_net() {
        let expected_move_white = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);
        let expected_move_black = Move::new(Square::F6, Square::G8, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

        let evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS);
        let mut optimizer = nn::Adam::default().build(&evaluator.model.vs, 0.005).unwrap();

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
            let random_batch_vec = get_random_batch_from_pgns(&pgns, 120, rng);
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

            train_loss_metrics = train_batch(&evaluator.model, &mut optimizer, &modified_random_batch_vec);

            println!(
                "Batch {}/{} Completed. Training (Policy: {:.4}, Value: {:.4}, Total: {:.4})",
                i + 1, 10,
                train_loss_metrics.policy_loss, train_loss_metrics.value_loss, train_loss_metrics.total_loss,
            );
        }

        assert!(train_loss_metrics.policy_loss < 0.1);
        assert!(train_loss_metrics.value_loss < 0.1);
        assert!(train_loss_metrics.total_loss < 0.1);

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
            let is_white = state.side_to_move == Color::White;
            let expected_move = if is_white { expected_move_white } else { expected_move_black };

            let legal_moves = state.calc_legal_moves();
            assert!(legal_moves.contains(&expected_move));

            let expected_value = if is_white { 1.0 } else { -1.0 };
            let evaluation = evaluator.evaluate(state);

            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert!(prob > 0.7, "Expected move: {:?}, Prob: {}", mv, prob);
                } else {
                    assert!(prob < 0.05, "Unexpected move: {:?}, Prob: {}", mv, prob);
                }
            }

            assert_eq!(evaluation.value, expected_value);
        }
    }
}