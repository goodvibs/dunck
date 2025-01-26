use tch::{nn, Tensor};
use tch::nn::{Module, ModuleT};
use crate::engine::evaluators::neural::training_utils::print_tensor_stats;

#[derive(Debug)]
pub struct ResidualBlock {
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    conv2: nn::Conv2D,
    bn2: nn::BatchNorm,
    // se: SELayer,
}

impl ResidualBlock {
    pub fn new(root: &nn::Path, channels: i64) -> Self {
        let conv_config = nn::ConvConfig {
            padding: 1,
            ..Default::default()
        };

        ResidualBlock {
            conv1: nn::conv2d(root, channels, channels, 3, conv_config),
            bn1: nn::batch_norm2d(root, channels, Default::default()),
            conv2: nn::conv2d(root, channels, channels, 3, conv_config),
            bn2: nn::batch_norm2d(root, channels, Default::default()),
            // se: SELayer::new(vs, channels, 32),  // 32 is typical SE_CHANNELS value
        }
    }

    pub fn forward_t(&self, x: &Tensor, train: bool) -> Tensor {
        let residual = x;

        // First conv block
        let mut out = self.conv1.forward_t(x, train);
        
        out = self.bn1.forward_t(&out, train).relu();
        
        out = self.conv2.forward_t(&out, train);
        
        out = self.bn2.forward_t(&out, train);
        
        out = (out + residual).relu();

        out
    }
}