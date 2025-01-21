use tch::{nn, Tensor};
use crate::engine::evaluators::neural::constants::NUM_TARGET_SQUARE_POSSIBILITIES;

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

    pub fn forward(&self, x: &Tensor, train: bool) -> Tensor {
        x.apply(&self.conv1)
            .apply_t(&self.bn, train)
            .apply(&self.conv2)
            .view([-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64])
    }
}