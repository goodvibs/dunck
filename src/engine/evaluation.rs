use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;

pub fn get_value_at_terminal_state(state: &State, for_color: Color) -> f64 {
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

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub policy: Vec<(Move, f64)>,
    pub value: f64,
}

pub trait Evaluator {
    fn evaluate(&self, state: &State) -> Evaluation;
}