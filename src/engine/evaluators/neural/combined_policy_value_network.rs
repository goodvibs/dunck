use tch::Tensor;

pub trait CombinedPolicyValueNetwork {
    fn forward(&self, input: &Tensor, train: bool) -> (Tensor, Tensor);
}