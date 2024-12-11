use dunck::engine::conv_net_evaluator::constants::{NUM_OUTPUT_POLICY_MOVES, NUM_TARGET_SQUARE_POSSIBILITIES};
use dunck::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor};
use dunck::engine::conv_net_evaluator::ConvNetEvaluator;
use dunck::engine::evaluation::Evaluation;
use dunck::pgn::PgnStateTree;
use dunck::r#move::MoveFlag;
use dunck::state::{State, Termination};
use dunck::utils::Color;
use rand::distributions::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use std::fs::exists;
use std::str::FromStr;
use std::time::Instant;
use tch::{nn, Kind, Tensor};
use tch::nn::OptimizerConfig;

pub const MULTI_PGN_FILE: &str = "data/lichess_elite_db_multi_pgn/accepted.pgn";
pub const MODEL_FILE: &str = "model.safetensors";

pub const NUM_RESIDUAL_BLOCKS: usize = 10;
pub const NUM_FILTERS: i64 = 256;
pub const DROPOUT: f64 = 0.3;

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
    let mut evaluator = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS, DROPOUT, true);
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
    let mut evaluator2 = ConvNetEvaluator::new(NUM_RESIDUAL_BLOCKS, NUM_FILTERS, DROPOUT, true);
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

fn get_training_data_for_epoch(
    pgns: &[String],
    num_batches_per_epoch: usize,
    random_state: &mut ThreadRng,
) -> Vec<(State, Evaluation)> {
    let mut training_data = Vec::new();
    for _ in 0..num_batches_per_epoch {
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

            let training_example = match get_random_example_from_state_tree(state_tree, random_state) {
                Some(example) => example,
                None => continue,
            };

            training_data.push(training_example);
            break;
        }
    }
    training_data
}

fn train(
    pgns: &[String],
    num_epochs: usize,
    num_batches_per_epoch: usize,
    learning_rate: f64,
) {
    let mut random_state = rand::thread_rng();
    let mut evaluator = load_evaluator();

    let mut optimizer = nn::Adam::default()
        .build(&evaluator.model.vs, learning_rate)
        .expect("Failed to create optimizer");

    let start_time = Instant::now();

    for epoch in 0..num_epochs {
        println!("Starting epoch {}/{}", epoch + 1, num_epochs);
        let training_data = get_training_data_for_epoch(pgns, num_batches_per_epoch, &mut random_state);
        train_epoch(&mut evaluator, &mut optimizer, &training_data);
    }

    println!("Training completed. Total time elapsed: {:.2}s", start_time.elapsed().as_secs_f32());
    verify_and_save_model(&evaluator);
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
    if num_moves < 40 {
        return None;
    }

    let weights: Vec<f64> = (0..num_moves).map(|i| (i + 1) as f64).collect();
    let weight_sum: f64 = weights.iter().sum();
    let probabilities: Vec<f64> = weights.iter().map(|w| w / weight_sum).collect();

    let dist = rand::distributions::WeightedIndex::new(&probabilities).unwrap();
    let node_idx = dist.sample(rng);

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
                policy[policy_index as usize] = *prob;
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

fn run_batch(
    evaluator: &mut ConvNetEvaluator,
    optimizer: &mut nn::Optimizer,
    batch_data: &[(State, Evaluation)]
) {
    let (states, policies, values) = create_batch_tensors(batch_data);

    let (pred_policies, pred_values) = evaluator.model.forward(&states, true);

    // Compute policy cross-entropy loss
    let log_probs = pred_policies.log_softmax(-1, Kind::Float);
    let policy_loss = -(policies * &log_probs)
        .sum_dim_intlist(&[1i64, 2i64, 3i64][..], false, Kind::Float)
        .mean(Kind::Float);

    // Compute value MSE loss
    let value_loss = pred_values.mse_loss(&values, tch::Reduction::Mean);

    // Total loss
    let total_loss = &policy_loss + &value_loss;

    optimizer.zero_grad();
    total_loss.backward();
    optimizer.step();

    let policy_loss_scalar = policy_loss.double_value(&[]);
    let value_loss_scalar = value_loss.double_value(&[]);
    let total_loss_scalar = total_loss.double_value(&[]);

    println!(
        "Loss - Policy: {:.4}, Value: {:.4}, Total: {:.4}",
        policy_loss_scalar, value_loss_scalar, total_loss_scalar
    );
}

fn train_epoch(
    evaluator: &mut ConvNetEvaluator,
    optimizer: &mut nn::Optimizer,
    training_data: &[(State, Evaluation)],
) {
    let mut indices: Vec<usize> = (0..training_data.len()).collect();
    indices.shuffle(&mut rand::thread_rng());

    // In this example, we treat the entire `training_data` as one batch.
    // If you need smaller batches, you can chunk further.
    for chunk in indices.chunks(training_data.len()) {
        let batch_data: Vec<_> = chunk.iter().map(|&i| training_data[i].clone()).collect();
        run_batch(evaluator, optimizer, &batch_data);
    }
}

fn main() {
    let multi_pgn_file_content = std::fs::read_to_string(MULTI_PGN_FILE).expect("Failed to read PGN file");
    let pgns = extract_pgns(&multi_pgn_file_content);

    for i in 0..200 {
        let learning_rate = 0.0001;
        println!("|*| Training iteration {}/200 with learning rate {} |*|", i + 1, learning_rate);
        train(&pgns, 50, 64, learning_rate);
    }
}