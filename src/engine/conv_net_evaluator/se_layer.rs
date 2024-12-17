use tch::{nn, Kind, Tensor};

#[derive(Debug)]
pub struct SELayer {
    fc1: nn::Linear,
    fc2: nn::Linear,
}

impl SELayer {
    pub fn new(vs: &nn::Path, channels: i64, se_channels: i64) -> Self {
        SELayer {
            fc1: nn::linear(vs, channels, se_channels, Default::default()),
            fc2: nn::linear(vs, se_channels, 2 * channels, Default::default()),
        }
    }

    pub fn forward(&self, x: &Tensor) -> Tensor {
        let batch_size = x.size()[0];

        // Global average pooling
        let se = x.mean_dim(&[-2, -1][..], false, Kind::Float)  // Average over spatial dimensions
            .view([batch_size, -1])
            .apply(&self.fc1)
            .relu()
            .apply(&self.fc2);

        let chunks = se.chunk(2, 1);  // Split into two parts along channel dimension
        let w = &chunks[0];
        let b = &chunks[1];

        let z = w.sigmoid();
        x * z.view([-1, z.size()[1], 1, 1]) + b.view([-1, b.size()[1], 1, 1])
    }
}