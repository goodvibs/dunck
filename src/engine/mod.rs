use crate::state::{State, Termination};
use crate::utils::Color;

pub mod neural_network;
pub mod mcts;
pub mod material_evaluator;
pub mod rollout_evaluator;

fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
    match state.termination.unwrap() {
        Termination::Checkmate => {
            let checkmated_side = state.side_to_move;
            if checkmated_side == for_color {
                -1.
            } else {
                1.
            }
        }
        _ => 0.
    }
}
