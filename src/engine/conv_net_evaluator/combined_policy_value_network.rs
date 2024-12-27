use tch::Tensor;

pub trait CombinedPolicyValueNetwork {
    fn forward(&self, x: &Tensor, train: bool) -> (Tensor, Tensor);
}