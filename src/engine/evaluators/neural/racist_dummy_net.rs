use tch::{Kind, Tensor};
use crate::engine::evaluators::neural::combined_policy_value_network::CombinedPolicyValueNetwork;
use crate::engine::evaluators::neural::constants::{NUM_BITS_PER_BOARD, NUM_POSITION_BITS, NUM_TARGET_SQUARE_POSSIBILITIES};
use crate::engine::evaluators::neural::utils::DEVICE;

pub struct RacistDummyNet {
    pub white_value_output: f64,
    pub black_value_output: f64,
    pub white_policy_output: Tensor,
    pub black_policy_output: Tensor,
}

impl CombinedPolicyValueNetwork for RacistDummyNet {
    fn forward_t(&self, input: &Tensor, train: bool) -> (Tensor, Tensor) {
        assert_eq!(input.size().len(), 4);
        assert_eq!(input.size()[1..4], [NUM_POSITION_BITS as i64, 8, 8]);
        let batch_size = input.size()[0];
        assert!(batch_size > 0);

        // Create policy tensor initialized to very small values
        let mut policy_logits = Tensor::zeros(&[batch_size, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
        let _ = policy_logits.fill_(-1.);  // Fill with very small values

        let value = Tensor::zeros(&[batch_size, 1], (Kind::Float, *DEVICE));

        // Get the side to move from the input tensor (channel 12)
        let side_to_move_channel = input.narrow(1, NUM_BITS_PER_BOARD as i64, 1);  // Get the side-to-move channel

        for i in 0..batch_size {
            let side_to_move_values = side_to_move_channel.get(i);
            let side_to_move_values_max = side_to_move_values.max().double_value(&[]);
            let side_to_move_values_min = side_to_move_values.min().double_value(&[]);
            assert_eq!(side_to_move_values_min, side_to_move_values_max);
            
            let is_white = match side_to_move_values_max {
                1. => true,
                0. => false,
                _ => panic!("Invalid side-to-move value: {}", side_to_move_values_max),
            };

            if is_white {
                policy_logits.get(i).copy_(&self.white_policy_output);
                let _ = value.get(i).fill_(self.white_value_output);
            } else {
                policy_logits.get(i).copy_(&self.black_policy_output);
                let _ = value.get(i).fill_(self.black_value_output);
            }
        }

        assert_eq!(policy_logits.size(), [batch_size, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        assert_eq!(value.size(), [batch_size, 1]);

        (policy_logits, value)
    }
}