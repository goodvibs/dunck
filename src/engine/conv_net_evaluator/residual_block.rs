use tch::{nn, Tensor};

// Define a Residual Block
#[derive(Debug)]
pub struct ResidualBlock {
    conv1: nn::Conv2D,
    bn1: nn::BatchNorm,
    conv2: nn::Conv2D,
    bn2: nn::BatchNorm,
}

impl ResidualBlock {
    pub fn new(vs: &nn::Path, channels: i64) -> ResidualBlock {
        // Initialize two convolutional layers with ReLU activations
        let conv1 = nn::conv2d(
            vs,
            channels,
            channels,
            3,
            nn::ConvConfig { padding: 1, ..Default::default() }
        );

        let bn1 = nn::batch_norm2d(vs, channels, Default::default());

        let conv2 = nn::conv2d(
            vs,
            channels,
            channels,
            3,
            nn::ConvConfig { padding: 1, ..Default::default() }
        );

        let bn2 = nn::batch_norm2d(vs, channels, Default::default());

        ResidualBlock {
            conv1,
            bn1,
            conv2,
            bn2,
        }
    }

    // Forward pass for the residual block
    pub fn forward(&self, x: &Tensor, train: bool) -> Tensor {
        let residual = x;  // Save the input for the skip connection
        let x = x.apply(&self.conv1).apply_t(&self.bn1, train).relu();
        let x = x.apply(&self.conv2).apply_t(&self.bn2, train);
        x + residual  // Skip connection
    }
}