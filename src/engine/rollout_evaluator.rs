use rand::prelude::SliceRandom;
use crate::engine::evaluate_terminal_state;
use crate::engine::mcts::{Evaluation, Evaluator};
use crate::state::State;
use crate::utils::Color;

#[derive(Clone)]
pub struct RolloutEvaluator {
    pub num_rollouts: u32,
}

impl RolloutEvaluator {
    pub fn new(num_rollouts: u32) -> Self {
        Self {
            num_rollouts,
        }
    }
}

impl Evaluator for RolloutEvaluator {
    fn evaluate(&self, state: &State) -> Evaluation {
        let initial_moves = state.calc_legal_moves();
        let side_to_move = state.side_to_move;
        let mut state = state.clone();
        let mut rng = rand::thread_rng();
        let mut i = 0;
        let value;
        loop {
            let moves = state.calc_legal_moves();
            if moves.is_empty() {
                state.assume_and_update_termination();
                value = evaluate_terminal_state(&state, side_to_move);
                break;
            } else {
                let mv = moves.choose(&mut rng).unwrap();
                state.make_move(*mv);
            }
            i += 1;
            
            if i >= self.num_rollouts {
                value = 0.;
                break;
            }
        }
        
        let mut policy = Vec::with_capacity(initial_moves.len());
        for mv in initial_moves.iter() {
            policy.push((*mv, 1. / initial_moves.len() as f64));
        }
        
        Evaluation {
            policy,
            value,
        }
    }
}