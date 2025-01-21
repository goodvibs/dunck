use tch::{nn, Tensor};

#[derive(Debug)]
pub struct ResidualBlock {
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    conv2: nn::Conv2D,
    bn2: nn::BatchNorm,
    // se: SELayer,
}

impl ResidualBlock {
    pub fn new(vs: &nn::Path, channels: i64) -> Self {
        let conv_config = nn::ConvConfig {
            padding: 1,
            ..Default::default()
        };

        ResidualBlock {
            conv1: nn::conv2d(vs, channels, channels, 3, conv_config),
            bn1: nn::batch_norm2d(vs, channels, Default::default()),
            conv2: nn::conv2d(vs, channels, channels, 3, conv_config),
            bn2: nn::batch_norm2d(vs, channels, Default::default()),
            // se: SELayer::new(vs, channels, 32),  // 32 is typical SE_CHANNELS value
        }
    }

    pub fn forward(&self, x: &Tensor, train: bool) -> Tensor {
        let residual = x;

        // First conv block
        let out = x.apply(&self.conv1)
            .apply_t(&self.bn1, train)
            .relu();

        // Second conv block
        let out = out.apply(&self.conv2)
            .apply_t(&self.bn2, train)
            .relu();

        // Apply SE layer
        // let out = self.se.forward(&out);

        // Add residual connection and apply ReLU
        (out + residual).relu()
    }
}