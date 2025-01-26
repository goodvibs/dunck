use tch::{nn, Kind, Tensor};
use tch::nn::ModuleT;
use crate::engine::evaluators::neural::constants::NUM_TARGET_SQUARE_POSSIBILITIES;
use crate::engine::evaluators::neural::training_utils::print_tensor_stats;

#[derive(Debug)]
pub struct PolicyHead {
    conv1: nn::Conv2D,
    bn: nn::BatchNorm,
    conv2: nn::Conv2D,
}

impl PolicyHead {
    pub fn new(vs: &nn::Path, num_filters: i64) -> Self {
        PolicyHead {
            conv1: nn::conv2d(vs, num_filters, num_filters, 3, nn::ConvConfig { padding: 1, ..Default::default() }),
            bn: nn::batch_norm2d(vs, num_filters, Default::default()),
            conv2: nn::conv2d(vs, num_filters, NUM_TARGET_SQUARE_POSSIBILITIES as i64, 3, nn::ConvConfig { padding: 1, ..Default::default() }),
        }
    }

    pub fn forward_t(&self, x: &Tensor, train: bool) -> Tensor {
        print_tensor_stats(x, "PolicyHead input");
        
        let mut out = self.conv1.forward_t(x, train);
        print_tensor_stats(&out, "After conv1");
        
        out = self.bn.forward_t(&out, train).relu();
        print_tensor_stats(&out, "After bn+relu");
        
        out = self.conv2.forward_t(&out, train);
        
        out = out.view([-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        
        print_tensor_stats(&out, "Policy output");
        
        out
    }
}