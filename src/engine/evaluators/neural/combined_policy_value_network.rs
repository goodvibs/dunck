use tch::Tensor;

pub trait CombinedPolicyValueNetwork {
    fn forward_t(&self, input: &Tensor, train: bool) -> (Tensor, Tensor);
}