use tch::{nn, Kind, Tensor};
use crate::engine::evaluation::Evaluation;
use crate::engine::evaluators::neural::combined_policy_value_network::CombinedPolicyValueNetwork;
use crate::engine::evaluators::neural::constants::{NUM_POSITION_BITS, NUM_TARGET_SQUARE_POSSIBILITIES};
use crate::engine::evaluators::neural::conv_net::ConvNet;
use crate::engine::evaluators::neural::utils::{state_to_tensor, PolicyIndex, DEVICE};
use crate::state::State;

pub struct LossMetrics {
    pub policy_loss: f64,
    pub value_loss: f64,
    pub total_loss: f64,
}

/// Helper function to calculate losses and optionally update the model
pub fn run_model(
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

    let policy_loss = predicted_policies.cross_entropy_loss::<Tensor>(&expected_policies, None, tch::Reduction::Mean, -100, 0.) * 1000.;

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
pub fn compute_loss(
    model: &dyn CombinedPolicyValueNetwork,
    batch_data: &[(State, Evaluation)],
) -> LossMetrics {
    run_model(model, None, batch_data)
}

/// Update the model parameters given a batch of training data
pub fn train_batch(
    model: &ConvNet,
    optimizer: &mut nn::Optimizer,
    batch_data: &[(State, Evaluation)],
) -> LossMetrics {
    run_model(model, Some(optimizer), batch_data)
}

/// Create batch tensors for states, policies, and values
pub fn create_batch_tensors(training_data: &[(State, Evaluation)]) -> (Tensor, Tensor, Tensor) {
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
            let policy_index = PolicyIndex::calc(mv, state.side_to_move);

            // Fill the tensor directly using indexing
            let _ = policy_tensor
                .get(policy_index.source_rank_index as i64)
                .get(policy_index.source_file_index as i64)
                .get(policy_index.move_index as i64)
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

#[cfg(test)]
mod tests {
    use engine::evaluation::Evaluator;
    use std::iter::zip;
    use tch::{nn, Kind, Tensor};
    use tch::nn::OptimizerConfig;
    use engine::evaluators::neural::utils::state_to_tensor;
    use r#move::{Move, MoveFlag};
    use state::State;
    use utils::{Square};
    use crate::*;
    use crate::engine::evaluation::Evaluation;
    use crate::engine::evaluators::neural::combined_policy_value_network::CombinedPolicyValueNetwork;
    use crate::engine::evaluators::neural::constants::NUM_TARGET_SQUARE_POSSIBILITIES;
    use crate::engine::evaluators::neural::conv_net_evaluator::ConvNetEvaluator;
    use crate::engine::evaluators::neural::racist_dummy_evaluator::RacistDummyEvaluator;
    use crate::engine::evaluators::neural::racist_dummy_net::RacistDummyNet;
    use crate::engine::evaluators::neural::training::{compute_loss, train_batch, LossMetrics};
    use crate::engine::evaluators::neural::training_utils::{extract_pgns, get_labeled_random_batch_from_pgns};
    use crate::engine::evaluators::neural::utils::{PolicyIndex, DEVICE};
    use crate::utils::Color;

    const NUM_RESIDUAL_BLOCKS: usize = 10;
    const NUM_FILTERS: i64 = 256;

    const MULTI_PGN_FILE: &str = "data/lichess_elite_db_multi_pgn/accepted.pgn";
    
    fn create_test_dummy_net(expected_move_white: &Move, expected_move_black: &Move) -> RacistDummyNet {
        RacistDummyNet {
            white_value_output: 1.0,
            black_value_output: -1.0,
            white_policy_output: {
                let mut policy_logits = Tensor::zeros(&[8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
                let _ = policy_logits.fill_(-1.);

                let policy_index = PolicyIndex::calc(&expected_move_white, Color::White);
                let _ = policy_logits
                    .get(policy_index.source_rank_index as i64)
                    .get(policy_index.source_file_index as i64)
                    .get(policy_index.move_index as i64)
                    .fill_(1000.0);
                
                policy_logits
            },
            black_policy_output: {
                let mut policy_logits = Tensor::zeros(&[8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
                let _ = policy_logits.fill_(-1.);

                let policy_index = PolicyIndex::calc(&expected_move_black, Color::Black);
                let _ = policy_logits
                    .get(policy_index.source_rank_index as i64)
                    .get(policy_index.source_file_index as i64)
                    .get(policy_index.move_index as i64)
                    .fill_(1000.0);

                policy_logits
            }
        }
    }
    
    #[test]
    fn test_compute_loss() {
        let expected_move_white = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);
        let expected_move_black = Move::new(Square::F6, Square::G8, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

        let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
        let pgns = extract_pgns(&multi_pgn_file_content);
        let rng = &mut rand::thread_rng();

        let model = create_test_dummy_net(
            &expected_move_white,
            &expected_move_black
        );

        println!("Computing loss for {} samples", 500);
        let labeled_batch = get_labeled_random_batch_from_pgns(&pgns, 500, rng);
        let relabeled_batch = labeled_batch.iter().map(|(state, _)| {
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

        let loss_metrics = compute_loss(&model, &relabeled_batch);

        println!(
            "Batch loss computed. Policy loss: {}, Value loss: {}, Total loss: {}",
            loss_metrics.policy_loss, loss_metrics.value_loss, loss_metrics.total_loss,
        );

        assert_eq!(loss_metrics.policy_loss, 0.);
        assert_eq!(loss_metrics.value_loss, 0.);
        assert_eq!(loss_metrics.total_loss, 0.);
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
    
        let evaluator = RacistDummyEvaluator {
            model: create_test_dummy_net(
                &expected_move_white,
                &expected_move_black
            ),
        };
    
        for state in test_states.iter() {
            let is_white = state.side_to_move == Color::White;
            let expected_move = if is_white { expected_move_white } else { expected_move_black };
    
            let legal_moves = state.calc_legal_moves();
            assert!(legal_moves.contains(&expected_move));
    
            let expected_value = if is_white { 1.0 } else { -1.0 };
            let evaluation = evaluator.evaluate(state);
    
            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert_eq!(prob, 1.0);
                } else {
                    assert_eq!(prob, 0.0);
                }
            }
    
            assert_eq!(evaluation.value, expected_value);
        }
    }

    #[test]
    fn test_training_conv_net_white() {
        let expected_move = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

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
            let random_batch_vec = get_labeled_random_batch_from_pgns(&pgns, 120, rng);
            let modified_random_batch_vec = random_batch_vec.iter().map(|(state, _)| {
                let modified_eval = Evaluation {
                    policy: vec![(expected_move, 1.0)],
                    value: 1.0,
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
        ];

        for state in test_states.iter() {
            let legal_moves = state.calc_legal_moves();
            assert!(legal_moves.contains(&expected_move));

            let evaluation = evaluator.evaluate(state);

            println!("State:\n{}", state.board);
            println!("Turn: {:?}", state.side_to_move);
            println!("Evaluation: {:?}\n", evaluation);

            assert!((evaluation.value - 1.0).abs() < 0.00001, "Expected value: {}, Actual value: {}", 1.0, evaluation.value);

            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert!(prob > 0.7, "Expected move: {:?}, Prob: {}", mv, prob);
                } else {
                    assert!(prob < 0.05, "Unexpected move: {:?}, Prob: {}", mv, prob);
                }
            }
        }
    }

    #[test]
    fn test_training_conv_net_black() {
        let expected_move = Move::new(Square::F6, Square::G8, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);

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

        let mut batch_num = 0;
        let mut patience_counter = 3;

        while patience_counter > 0 {
            println!("Starting batch {}", batch_num + 1);
            let random_batch_vec = get_labeled_random_batch_from_pgns(&pgns, 512, rng);
            let modified_random_batch_vec = random_batch_vec.iter().map(|(state, _)| {
                let modified_eval = Evaluation {
                    policy: vec![(expected_move, 1.0)],
                    value: -1.0,
                };
                (state.clone(), modified_eval)
            }).collect::<Vec<_>>();

            train_loss_metrics = train_batch(&evaluator.model, &mut optimizer, &modified_random_batch_vec);

            println!(
                "Batch {} Completed. Training (Policy: {:.4}, Value: {:.4}, Total: {:.4})",
                batch_num + 1,
                train_loss_metrics.policy_loss, train_loss_metrics.value_loss, train_loss_metrics.total_loss,
            );

            if train_loss_metrics.total_loss < 0.1 {
                patience_counter -= 1;
            }

            batch_num += 1;
        }

        assert!(train_loss_metrics.policy_loss < 0.1);
        assert!(train_loss_metrics.value_loss < 0.1);
        assert!(train_loss_metrics.total_loss < 0.1);

        let test_states = [
            // black to move
            State::from_fen("rnbqkbnr/ppp1pppp/8/3p4/5P2/5N2/PPPPP1PP/RNBQKB1R b KQkq - 1 2").unwrap(),
            State::from_fen("rnbqkbnr/1pp1pppp/8/p2p4/5P2/5N1P/PPPPP1P1/RNBQKB1R b KQkq - 0 3").unwrap(),
            State::from_fen("rnbqkbnr/2p1pppp/8/1N1p4/1p3P1P/8/P1PPP1P1/R1BQKBNR b KQkq - 0 5").unwrap(),
        ];

        for state in test_states.iter() {
            let legal_moves = state.calc_legal_moves();
            assert!(legal_moves.contains(&expected_move));

            let evaluation = evaluator.evaluate(state);

            println!("State:\n{}", state.board);
            println!("Turn: {:?}", state.side_to_move);
            println!("Evaluation: {:?}\n", evaluation);

            assert!((evaluation.value - (-1.0)).abs() < 0.0001, "Expected value: {}, Actual value: {}", -1.0, evaluation.value);

            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert!(prob > 0.4, "Expected move: {:?}, Prob: {}", mv, prob);
                } else {
                    assert!(prob < 0.05, "Unexpected move: {:?}, Prob: {}", mv, prob);
                }
            }
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
            let random_batch_vec = get_labeled_random_batch_from_pgns(&pgns, 120, rng);
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

            println!("State:\n{}", state.board);
            println!("Turn: {:?}", state.side_to_move);
            println!("Evaluation: {:?}\n", evaluation);

            assert!((evaluation.value - expected_value).abs() < 0.00001, "Expected value: {}, Actual value: {}", expected_value, evaluation.value);

            for (mv, prob) in evaluation.policy {
                if mv == expected_move {
                    assert!(prob > 0.7, "Expected move: {:?}, Prob: {}", mv, prob);
                } else {
                    assert!(prob < 0.05, "Unexpected move: {:?}, Prob: {}", mv, prob);
                }
            }
        }
    }
}