mod neural_network;
pub(crate) mod mcts;
pub(crate) mod material_evaluator;

use std::cell::RefCell;
use std::cmp::max_by;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use rand::prelude::SliceRandom;
use rand::Rng;
use crate::engine::mcts::{Evaluation, Evaluator};
use crate::r#move::Move;
use crate::state::{Context, State, Termination};
use crate::utils::{Color, PieceType};

fn simulate_rollout(mut state: State, for_color: Color) -> f64 {
    const MAX_ROLLOUT_DEPTH: u32 = 50;
    let mut rng = rand::thread_rng();
    let mut i = 0;
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            state.assume_and_update_termination();
            return evaluate_terminal_state(&state, for_color);
        } else {
            // let rand_idx = rng.usize(..moves.len());
            // let mv = moves[rand_idx];
            let mv = moves.choose(&mut rng).unwrap();
            state.make_move(*mv);
        }
        i += 1;
    }
}

fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
    match state.termination.unwrap() {
        Termination::Checkmate => {
            let checkmated_side = state.side_to_move;
            if checkmated_side == for_color {
                0.
            } else {
                1.
            }
        }
        _ => 0.5
    }
}
