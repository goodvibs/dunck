use std::fs::exists;
use tch::nn::OptimizerConfig;
use tch::{nn, Device, Tensor};
use rand::seq::SliceRandom;
use std::time::Instant;
use crate::engine::conv_net_evaluator::constants::NUM_OUTPUT_POLICY_MOVES;
use crate::engine::conv_net_evaluator::ConvNetEvaluator;
use crate::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor};
use crate::engine::mcts::{Evaluation, MCTS};
use crate::r#move::MoveFlag;
use crate::state::State;

pub const EXPLORATION_PARAM: f64 = 1.5;
pub const NUM_RESIDUAL_BLOCKS: usize = 4;
pub const NUM_FILTERS: i64 = 8;
pub const BATCH_SIZE: i64 = 256;
pub const LEARNING_RATE: f64 = 0.001;
pub const GAMES_BEFORE_TRAINING: usize = 100;
pub const MAX_GAME_DEPTH: usize = 200;

fn train(num_games: usize, num_mcts_iterations_per_move: usize) {
    let mut evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS, true);

    if exists("model.pt").expect("Failed to check if model file exists") {
        println!("Loading model from file...");
        evaluator.model.load("model.pt").expect("Failed to load model");
    }

    let mut optimizer = nn::Adam::default()
        .build(&evaluator.model.vs, LEARNING_RATE)
        .expect("Failed to create optimizer");
    
    let mut all_training_data: Vec<(State, Evaluation)> = Vec::new();
    let start_time = Instant::now();

    for game_idx in 0..num_games {
        println!("Starting game {}/{}", game_idx + 1, num_games);
        
        // Create MCTS with save_data enabled
        let mut mcts = MCTS::new(State::initial(), EXPLORATION_PARAM, &evaluator, true);

        // Play game and collect training data
        mcts.play_game(num_mcts_iterations_per_move, MAX_GAME_DEPTH);

        // Get training data from MCTS
        all_training_data.extend(mcts.state_evaluations);

        // Train after collecting enough games
        if (game_idx + 1) % GAMES_BEFORE_TRAINING == 0 {
            println!("Training on {} positions", all_training_data.len());
            train_epoch(&mut evaluator, &mut optimizer, &all_training_data);

            // Save model checkpoint
            evaluator.model.save("model.pt").expect("Failed to save model");

            // Clear data after training
            all_training_data.clear();
        }

        // Log progress
        if (game_idx + 1) % 10 == 0 {
            let elapsed = start_time.elapsed();
            println!(
                "Completed {}/{} games. Time elapsed: {:.2}s",
                game_idx + 1,
                num_games,
                elapsed.as_secs_f32()
            );
        }
    }
}

fn train_epoch(
    evaluator: &mut ConvNetEvaluator,
    optimizer: &mut nn::Optimizer,
    training_data: &[(State, Evaluation)],
) {
    let mut indices: Vec<usize> = (0..training_data.len()).collect();
    indices.shuffle(&mut rand::thread_rng());

    for chunk in indices.chunks(BATCH_SIZE as usize) {
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
                Tensor::from_slice(&policy)
            })
            .collect();

        // Convert values to tensors
        let batch_values: Vec<_> = chunk
            .iter()
            .map(|&i| Tensor::from_slice(&[training_data[i].1.value]))
            .collect();

        // Stack into batch tensors
        let states = Tensor::stack(&batch_states, 0);
        let policies = Tensor::stack(&batch_policies, 0);
        let values = Tensor::stack(&batch_values, 0);

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