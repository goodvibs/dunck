use dunck::engine::evaluators::neural::conv_net_evaluator::ConvNetEvaluator;
use std::fs::exists;
use tch::nn::OptimizerConfig;
use tch::{nn, Tensor};
use dunck::engine::evaluators::neural::training::{compute_loss, train_batch};
use dunck::engine::evaluators::neural::training_utils::{extract_pgns, get_labeled_random_batch_from_pgns};

pub const MULTI_PGN_FILE: &str = "data/lichess_elite_db_multi_pgn/accepted.pgn";
pub const MODEL_FILE: &str = "model.safetensors";

pub const NUM_RESIDUAL_BLOCKS: usize = 10;
pub const NUM_FILTERS: i64 = 256;

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

    let validation_data = get_labeled_random_batch_from_pgns(&pgns, num_examples_per_batch, &mut random_state);

    for i in 0..num_iterations {
        println!("|*| Training iteration {}/{} with learning rate {} |*|", i + 1, num_iterations, learning_rate);

        let mut evaluator = load_evaluator();
        let mut optimizer = nn::Adam::default()
            .build(&evaluator.model.vs, learning_rate)
            .expect("Failed to create optimizer");

        for batch_num in 0..num_batches {
            println!("Starting batch {}/{}", batch_num + 1, num_batches);

            // Get fresh training data for this batch
            let training_data = get_labeled_random_batch_from_pgns(&pgns, num_examples_per_batch, &mut random_state);

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