use std::iter::zip;
use tch::{Kind, Tensor};
use crate::engine::evaluation::{Evaluation, Evaluator};
use crate::engine::evaluators::neural::combined_policy_value_network::CombinedPolicyValueNetwork;
use crate::engine::evaluators::neural::racist_dummy_net::RacistDummyNet;
use crate::engine::evaluators::neural::utils::{state_to_tensor, PolicyIndex, DEVICE};
use crate::state::State;

pub struct RacistDummyEvaluator {
    pub model: RacistDummyNet
}

impl Evaluator for RacistDummyEvaluator {
    fn evaluate(&self, state: &State) -> Evaluation {
        let state_tensor = state_to_tensor(state);
        let input_tensor = Tensor::stack(&[state_tensor], 0).to(*DEVICE); // No batch, so stack along the first dimension
        let (policy_logits, value_tensor) = self.model.forward_t(&input_tensor, false);
        
        let legal_moves = state.calc_legal_moves();
        let legal_moves_policy_logits = Tensor::zeros(&[legal_moves.len() as i64], (Kind::Float, *DEVICE));

        for (i, mv) in legal_moves.iter().enumerate() {
            let policy_index = PolicyIndex::calc(mv, state.side_to_move);

            let policy_logit = policy_logits.double_value(&[
                0,
                policy_index.source_rank_index as i64,
                policy_index.source_file_index as i64,
                policy_index.move_index as i64
            ]);

            let _ = legal_moves_policy_logits.get(i as i64).fill_(policy_logit);
        }

        let priors = legal_moves_policy_logits.softmax(-1, Kind::Float).to_device(*DEVICE);
        let priors_vec = Vec::<f32>::try_from(priors).unwrap();
        
        let policy = zip(legal_moves, priors_vec)
            .map(|(mv, prior)| (mv.clone(), prior as f64))
            .collect();

        Evaluation {
            policy,
            value: value_tensor.double_value(&[]),
        }
    }
}