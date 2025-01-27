use std::error::Error;
use tch::{nn, Device, Kind, Tensor};
use tch::nn::{ModuleT};
use crate::engine::evaluators::neural::constants::*;
use crate::engine::evaluators::neural::combined_policy_value_network::CombinedPolicyValueNetwork;
use crate::engine::evaluators::neural::policy_head::PolicyHead;
use crate::engine::evaluators::neural::residual_block::ResidualBlock;
use crate::engine::evaluators::neural::training_utils::print_tensor_stats;
use crate::engine::evaluators::neural::value_head::ValueHead;

// Define the main model structure
#[derive(Debug)]
pub struct ConvNet {
    pub vs: nn::VarStore,
    pub num_filters: i64,
    pub conv1: nn::Conv2D,
    pub bn1: nn::BatchNorm,
    pub residual_blocks: Vec<ResidualBlock>,
    pub policy_head: PolicyHead,
    pub value_head: ValueHead,
}

impl ConvNet {
    pub fn new(device: Device, num_residual_blocks: usize, num_filters: i64) -> ConvNet {
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

        let policy_head = PolicyHead::new(root, num_filters);
        let value_head = ValueHead::new(root, num_filters);

        ConvNet {
            vs,
            num_filters,
            conv1,
            bn1,
            residual_blocks,
            policy_head,
            value_head,
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

        // After network creation
        for (name, tensor) in self.vs.variables() {
            println!("Layer {}: sum = {}, mean = {}, std = {}",
                     name,
                     tensor.sum(Kind::Float).double_value(&[]),
                     tensor.mean(Kind::Float).double_value(&[]),
                     tensor.std(true).double_value(&[])
            );
        }
        
        Ok(())
    }
}

impl CombinedPolicyValueNetwork for ConvNet {
    /// Forward pass through the model
    fn forward_t(&self, x: &Tensor, train: bool) -> (Tensor, Tensor) {
        assert_eq!(x.size().len(), 4);
        assert_eq!(x.size()[1..4], [NUM_POSITION_BITS as i64, 8, 8]);
        assert!(x.size()[0] > 0);

        // Debug print initial tensor
        print_tensor_stats(x, "Initial tensor");
        
        // Apply initial convolution, batch normalization, and ReLU activation
        let mut x = self.conv1.forward_t(x, train);
        print_tensor_stats(&x, "After conv1");
        
        x = self.bn1.forward_t(&x, train).relu();
        print_tensor_stats(&x, "After bn1+relu");

        // Pass through the residual blocks
        for block in &self.residual_blocks {
            x = block.forward_t(&x, train);
        }
        print_tensor_stats(&x, "After residual blocks");

        // Should be batch_size x 8 x 8 x 73
        let policy = self.policy_head.forward_t(&x, train);
        // Should be batch_size x 1
        let value = self.value_head.forward_t(&x, train);
        
        assert_eq!(policy.size().len(), value.size().len() + 2);

        (policy, value)
    }
}

#[cfg(test)]
mod tests {
    use tch::Kind;
    use tch::nn::OptimizerConfig;
    use crate::engine::evaluators::neural::utils::{state_to_tensor, DEVICE};
    use crate::state::State;
    use super::*;

    #[test]
    fn test_chess_model() {
        let model = ConvNet::new(*DEVICE, 10, 256);

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward_t(&input_tensor, false);

        assert_eq!(policy.size(), [1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        assert_eq!(value.size(), [1, 1]);
    }

    #[test]
    fn test_training() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ConvNet::new(*DEVICE, 10, 256);

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward_t(&input_tensor, true);

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
        let model = ConvNet::new(*DEVICE, 10, 256);
        let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();

        for _ in 0..1000 {
            let input_tensor = state_to_tensor(&State::initial());
            let (policy, value) = model.forward_t(&input_tensor, true);

            let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
            let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

            let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
            let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);
            let loss = policy_loss + value_loss;

            optimizer.backward_step(&loss);
        }
    }
}