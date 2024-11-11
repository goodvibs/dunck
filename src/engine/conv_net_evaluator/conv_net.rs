use std::error::Error;
use tch::{nn, Device, Kind, Tensor};
use tch::nn::Module;
use crate::engine::conv_net_evaluator::constants::*;
use crate::engine::conv_net_evaluator::residual_block::ResidualBlock;
use crate::engine::conv_net_evaluator::utils::{is_knight_jump, DEVICE};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenLikeMoveDirection, Square};

// Define the main model structure
#[derive(Debug)]
pub struct ConvNet {
    pub vs: nn::VarStore,
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    residual_blocks: Vec<ResidualBlock>,
    fc_policy: nn::Linear,
    fc_value: nn::Linear,
}

impl ConvNet {
    pub fn new(device: Device, num_residual_blocks: usize) -> ConvNet {
        let vs = nn::VarStore::new(device);
        let root = &vs.root();
        
        // Initial convolutional layer
        let conv1 = nn::conv2d(root, NUM_POSITION_BITS as i64, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64, 3, nn::ConvConfig { padding: 1, ..Default::default() }); // 17 input channels, 32 output channels

        // Batch normalization for initial convolution layer
        let bn1 = nn::batch_norm2d(root, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64, Default::default());

        // Residual blocks
        let mut residual_blocks = Vec::new();
        for _ in 0..num_residual_blocks {
            residual_blocks.push(ResidualBlock::new(root, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64));
        }

        // Fully connected layers for policy and value heads
        let fc_policy = nn::linear(
            root,
            NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 64,
            NUM_OUTPUT_POLICY_MOVES as i64,
            Default::default(),
        );
        let fc_value = nn::linear(
            root,
            NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 64,
            1,
            Default::default(),
        );

        ConvNet {
            vs,
            conv1,
            bn1,
            residual_blocks,
            fc_policy,
            fc_value,
        }
    }

    /// Save model weights to a file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.vs.save(path)?;
        Ok(())
    }

    /// Load model weights from a file
    pub fn load(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        self.vs.load(path)?;
        Ok(())
    }

    /// Forward pass through the model
    pub fn forward(&self, x: &Tensor, train: bool) -> (Tensor, Tensor) {
        // Apply initial convolution, batch normalization, and ReLU activation
        let mut x = x.view([-1, NUM_POSITION_BITS as i64, 8, 8]).apply(&self.conv1);
        x = x.apply_t(&self.bn1, train).relu();

        // Pass through the residual blocks
        for block in &self.residual_blocks {
            x = block.forward(&x, train);
        }

        // Flatten for fully connected layers
        x = x.view([-1, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 8 * 8]);

        // Policy head: Softmax over 4672 possible moves
        let policy = self
            .fc_policy
            .forward(&x)
            .view([-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64])
            .softmax(-1, Kind::Float); // Softmax for move probabilities

        // Value head: Tanh for output between -1 and 1
        let value = self.fc_value.forward(&x).tanh();

        (policy, value)
    }
}

#[cfg(test)]
mod tests {
    use tch::nn::OptimizerConfig;
    use crate::engine::conv_net_evaluator::utils::state_to_tensor;
    use super::*;

    #[test]
    fn test_chess_model() {
        let model = ConvNet::new(*DEVICE, 4);

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor, false);

        assert_eq!(policy.size(), [1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        assert_eq!(value.size(), [1, 1]);
    }

    #[test]
    fn test_training() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ConvNet::new(*DEVICE, 4);

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor, true);

        let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
        let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

        let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
        let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);

        let loss = policy_loss + value_loss;

        let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();
        optimizer.backward_step(&loss);
    }

    #[test]
    fn test_train_1000_iterations() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ConvNet::new(*DEVICE, 4);
        let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();

        for _ in 0..1000 {
            let input_tensor = state_to_tensor(&State::initial());
            let (policy, value) = model.forward(&input_tensor, true);

            let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
            let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

            let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
            let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);
            let loss = policy_loss + value_loss;

            optimizer.backward_step(&loss);
        }
    }
}