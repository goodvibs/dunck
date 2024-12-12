use std::error::Error;
use tch::{nn, Device, Kind, Tensor};
use tch::nn::Module;
use crate::engine::conv_net_evaluator::constants::*;
use crate::engine::conv_net_evaluator::residual_block::ResidualBlock;

// Define the main model structure
#[derive(Debug)]
pub struct ConvNet {
    pub vs: nn::VarStore,
    pub num_filters: i64,
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    residual_blocks: Vec<ResidualBlock>,
    dropout: f64,
    fc_policy: nn::Linear,
    fc_value: nn::Linear,
}

impl ConvNet {
    pub fn new(device: Device, num_residual_blocks: usize, num_filters: i64, dropout: f64) -> ConvNet {
        let vs = nn::VarStore::new(device);
        let root = &vs.root();

        // Initial convolutional layer
        let conv1 = nn::conv2d(root, NUM_POSITION_BITS as i64, num_filters, 3, nn::ConvConfig { padding: 1, ..Default::default() }); // 17 input channels, num_filters output channels

        // Batch normalization for initial convolution layer
        let bn1 = nn::batch_norm2d(root, num_filters, Default::default());

        // Residual blocks
        let mut residual_blocks = Vec::new();
        for _ in 0..num_residual_blocks {
            residual_blocks.push(ResidualBlock::new(root, num_filters));
        }

        // Fully connected layers for policy and value heads
        let fc_policy = nn::linear(
            root,
            num_filters * 64,
            NUM_OUTPUT_POLICY_MOVES as i64,
            Default::default(),
        );
        let fc_value = nn::linear(
            root,
            num_filters * 64,
            1,
            Default::default(),
        );

        ConvNet {
            vs,
            num_filters,
            conv1,
            bn1,
            residual_blocks,
            dropout,
            fc_policy,
            fc_value,
        }
    }

    /// Save model weights manually using read_safetensors
    pub fn save(&self, path: &str) -> Result<(), Box<dyn Error>> {
        self.vs.save(path)?;
        Ok(())
    }

    /// Load model weights manually using fill_safetensors
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
        x = x.flatten(1, -1);

        if train {
            x = x.dropout(self.dropout, train);
        }

        let policy = self
            .fc_policy
            .forward(&x)
            .view([-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);

        // Value head: Tanh for output between -1 and 1
        let value = self.fc_value.forward(&x).tanh();

        (policy, value)
    }
}

#[cfg(test)]
mod tests {
    use tch::nn::OptimizerConfig;
    use crate::engine::conv_net_evaluator::utils::{state_to_tensor, DEVICE};
    use crate::state::State;
    use super::*;

    #[test]
    fn test_chess_model() {
        let model = ConvNet::new(*DEVICE, 10, 256, 0.3);

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor, false);

        assert_eq!(policy.size(), [1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        assert_eq!(value.size(), [1, 1]);
    }

    #[test]
    fn test_training() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ConvNet::new(*DEVICE, 10, 256, 0.3);

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
        let model = ConvNet::new(*DEVICE, 10, 256, 0.3);
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