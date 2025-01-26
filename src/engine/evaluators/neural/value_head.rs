use tch::{nn, Kind, Tensor};
use tch::nn::ModuleT;
use crate::engine::evaluators::neural::training_utils::print_tensor_stats;

#[derive(Debug)]
pub struct ValueHead {
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    conv2: nn::Conv2D,
    bn2: nn::BatchNorm,
    fc: nn::Linear,
}

impl ValueHead {
    pub fn new(vs: &nn::Path, num_filters: i64) -> Self {
        ValueHead {
            conv1: nn::conv2d(vs, num_filters, 32, 3, nn::ConvConfig { padding: 1, ..Default::default() }),
            bn1: nn::batch_norm2d(vs, 32, Default::default()),
            conv2: nn::conv2d(vs, 32, 128, 8, nn::ConvConfig { padding: 0, ..Default::default() }),
            bn2: nn::batch_norm1d(vs, 128, Default::default()),
            fc: nn::linear(vs, 128, 1, Default::default()),
        }
    }

    pub fn forward_t(&self, x: &Tensor, train: bool) -> Tensor {
        print_tensor_stats(x, "ValueHead input");
        
        let mut out = self.conv1.forward_t(x, train);
        print_tensor_stats(&out, "After conv");
        
        out = self.bn1.forward_t(&out, train).relu();
        print_tensor_stats(&out, "After first bn+relu");
        
        out = self.conv2.forward_t(&out, train);
        print_tensor_stats(&out, "After second conv");

        out = out.flatten(1, -1);
        
        out = self.bn2.forward_t(&out, train).relu();
        print_tensor_stats(&out, "After second bn+relu");
        
        out = self.fc.forward_t(&out, train).tanh();
        print_tensor_stats(&out, "Value output");

        out
    }
}