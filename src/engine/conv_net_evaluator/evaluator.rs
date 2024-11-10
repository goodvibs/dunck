use std::iter::zip;
use lazy_static::lazy_static;
use tch::{nn, nn::Module, nn::OptimizerConfig, Tensor, Device, Kind};
use crate::engine::conv_net_evaluator::conv_net::{ConvNet};
use crate::engine::conv_net_evaluator::utils::{get_policy_index_for_move, state_to_tensor, DEVICE};
use crate::engine::mcts::{Evaluation, Evaluator};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenLikeMoveDirection, Square};

pub struct ConvNetEvaluator {
    model: ConvNet,
}

impl ConvNetEvaluator {
    pub fn new() -> ConvNetEvaluator {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ConvNet::new(&vs.root(), 4);

        ConvNetEvaluator {
            model
        }
    }
}

impl Evaluator for ConvNetEvaluator {
    fn evaluate(&self, state: &State) -> Evaluation {
        let input_tensor = state_to_tensor(state);
        let (policy, value) = self.model.forward(&input_tensor);

        let legal_moves = state.calc_legal_moves();
        let mut priors = Vec::with_capacity(legal_moves.len());
        let mut sum = 0.;

        for mv in &legal_moves {
            let src_square_from_current_perspective = mv.get_source().to_perspective_from_white(state.side_to_move);
            let dst_square_from_current_perspective = mv.get_destination().to_perspective_from_white(state.side_to_move);
            let flag = mv.get_flag();
            let vetted_promotion = match flag {
                MoveFlag::Promotion => Some(mv.get_promotion()),
                _ => None
            };
            
            let policy_index = get_policy_index_for_move(
                src_square_from_current_perspective,
                dst_square_from_current_perspective,
                vetted_promotion,
                flag
            );
            
            let prior = policy.double_value(
                &[
                    0,
                    src_square_from_current_perspective.get_rank() as i64,
                    src_square_from_current_perspective.get_file() as i64,
                    policy_index as i64
                ]
            );
            
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
