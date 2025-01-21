use tch::{nn, Tensor};

#[derive(Debug)]
pub struct ValueHead {
    conv: nn::Conv2D,
    bn1: nn::BatchNorm,  // After conv
    fc1: nn::Linear,
    bn2: nn::BatchNorm,  // After fc1
    fc2: nn::Linear,
}

impl ValueHead {
    pub fn new(vs: &nn::Path, num_filters: i64) -> Self {
        ValueHead {
            conv: nn::conv2d(vs, num_filters, 32, 3, nn::ConvConfig { padding: 1, ..Default::default() }),
            bn1: nn::batch_norm2d(vs, 32, Default::default()),
            fc1: nn::linear(vs, 32 * 8 * 8, 128, Default::default()),
            bn2: nn::batch_norm1d(vs, 128, Default::default()),  // Note: regular batch_norm for fc layers
            fc2: nn::linear(vs, 128, 1, Default::default()),
        }
    }

    pub fn forward(&self, x: &Tensor, train: bool) -> Tensor {
        x.apply(&self.conv)
            .apply_t(&self.bn1, train)
            .relu()
            .flatten(1, -1)
            .apply(&self.fc1)
            .apply_t(&self.bn2, train)
            .relu()
            .apply(&self.fc2)
            .tanh()
    }
}