use std::iter::zip;
use crate::engine::conv_net_evaluator::conv_net::{ConvNet};
use crate::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor, DEVICE};
use crate::engine::evaluation::{Evaluation, Evaluator};
use crate::r#move::{MoveFlag};
use crate::state::State;

#[derive(Debug)]
pub struct ConvNetEvaluator {
    pub model: ConvNet,
    pub train: bool
}

impl ConvNetEvaluator {
    pub fn new(num_residual_blocks: usize, num_filters: i64, train: bool) -> ConvNetEvaluator {
        let model = ConvNet::new(*DEVICE, num_residual_blocks, num_filters);

        ConvNetEvaluator {
            model,
            train
        }
    }
}

impl Evaluator for ConvNetEvaluator {
    fn evaluate(&self, state: &State) -> Evaluation {
        let input_tensor = state_to_tensor(state);
        let (policy, value) = self.model.forward(&input_tensor, self.train);

        let legal_moves = state.calc_legal_moves();
        let mut priors = Vec::with_capacity(legal_moves.len());
        let mut sum = 0.;

        for mv in &legal_moves {
            let src_square_from_current_perspective = mv.get_source().to_perspective_from_white(state.side_to_move);
            let dst_square_from_current_perspective = mv.get_destination().to_perspective_from_white(state.side_to_move);
            let vetted_promotion = match mv.get_flag() {
                MoveFlag::Promotion => Some(mv.get_promotion()),
                _ => None
            };
            
            let policy_index = get_policy_index_for_move(
                src_square_from_current_perspective,
                dst_square_from_current_perspective,
                vetted_promotion
            );
            
            let prior = policy.double_value(&[
                0,
                src_square_from_current_perspective.get_rank() as i64,
                src_square_from_current_perspective.get_file() as i64,
                policy_index as i64
            ]).max(0.);
            
            priors.push(prior);
            sum += prior;
        }

        let policy = zip(legal_moves, priors)
            .map(|(mv, prior)| (mv, prior / sum))
            .collect();

        Evaluation {
            policy,
            value: value.double_value(&[]),
        }
    }
}
