mod evaluate;

use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::evaluate::evaluate_non_terminal_state;
use crate::r#move::Move;
use crate::state::{Context, State, Termination};
use crate::utils::Color;

const MAX_ROLLOUT_DEPTH: u32 = 10;

fn simulate_rollout(mut state: State) -> f64 {
    let maximize_for = state.side_to_move;
    let mut rng = fastrand::Rng::new();
    let mut i = 0;
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            return evaluate_terminal_state(&state, maximize_for);
        } else {
            let rand_idx = rng.usize(..moves.len());
            let mv = moves[rand_idx];
            state.make_move(mv);
        }
        i += 1;
        if i >= MAX_ROLLOUT_DEPTH {
            return evaluate_non_terminal_state(&state, maximize_for);
        }
    }
}

fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
    let termination = match &state.termination {
        Some(termination) => termination,
        None => &match state.board.is_color_in_check(state.side_to_move) {
            true => Termination::Checkmate,
            false => Termination::Stalemate,
        }
    };

    match termination {
        Termination::Checkmate => {
            let checkmated_side = state.side_to_move;
            if checkmated_side == for_color {
                -1.0
            } else {
                1.0
            }
        }
        _ => 0.0
    }
}

#[derive(Debug)]
struct MCTSNode {
    state_after_move: State,
    mv: Option<Move>,
    visits: u32,
    value: f64,
    children: Vec<*mut MCTSNode>,
}

impl MCTSNode {
    fn new(mv: Option<Move>, state_after_move: State) -> Self {
        Self {
            state_after_move,
            mv,
            visits: 0,
            value: 0.0,
            children: Vec::new(),
        }
    }

    fn run(&mut self, exploration_param: f64) -> f64 {
        let maximize_for = self.state_after_move.side_to_move;
        let possible_selected_child = self.select_child_with_ucb1(exploration_param);
        let value;

        match possible_selected_child {
            Some(selected_child) => unsafe {
                // Negate the child's value since it's from opponent's perspective
                value = (*selected_child).run(exploration_param);
            }
            None => unsafe {
                if self.visits == 0 {
                    value = simulate_rollout(self.state_after_move.clone());
                } else {
                    let legal_moves = self.state_after_move.calc_legal_moves();
                    for legal_move in legal_moves {
                        let mut new_state = self.state_after_move.clone();
                        new_state.make_move(legal_move);
                        let new_node = MCTSNode::new(Some(legal_move), new_state);
                        self.children.push(Box::into_raw(Box::new(new_node)));
                    }
                    if !self.children.is_empty() {
                        // Select a random child for first expansion
                        let random_idx = fastrand::usize(..self.children.len());
                        let random_child = self.children[random_idx].clone();
                        value = (*random_child).run(exploration_param);
                    }
                    else {
                        value = evaluate_terminal_state(&self.state_after_move, maximize_for);
                    }
                }
            }
        }
        self.visits += 1;
        self.value += value;
        value
    }

    fn calc_ucb1(&self, parent_visits: u32, exploration_param: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            self.value / self.visits as f64
                + exploration_param * ((parent_visits as f64).ln() / self.visits as f64).sqrt()
        }
    }

    fn select_child_with_ucb1(&mut self, exploration_param: f64) -> Option<*mut MCTSNode> {
        unsafe {
            self.children.iter().max_by(|a, b| {
                let a_ucb1 = (***a).calc_ucb1(self.visits, exploration_param);
                let b_ucb1 = (***b).calc_ucb1(self.visits, exploration_param);
                a_ucb1.partial_cmp(&b_ucb1).unwrap()
            }).cloned()
        }
    }
}

impl Drop for MCTSNode {
    fn drop(&mut self) {
        for child in self.children.iter() {
            unsafe {
                let _ = Box::from_raw(*child);
            }
        }
    }
}

struct MCTS {
    root: *mut MCTSNode,
    exploration_param: f64,
}

impl MCTS {
    fn new(state: State, exploration_param: f64) -> Self {
        Self {
            root: Box::into_raw(Box::new(MCTSNode::new(None, state))),
            exploration_param,
        }
    }

    fn run(&mut self, iterations: u32) {
        for _ in 0..iterations {
            unsafe { (*self.root).run(self.exploration_param) };
        }
    }

    fn select_best_move(&self) -> Option<*mut MCTSNode> {
        unsafe {
            (*self.root).children.iter().max_by(|a, b| {
                let a_score = (***a).value / (***a).visits as f64;
                let b_score = (***b).value / (***b).visits as f64;
                a_score.partial_cmp(&b_score).unwrap()
            }).cloned()
        }
    }
}

impl Drop for MCTS {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn test_mcts() {
        let exploration_param = 1.41;
        let mut mcts = MCTS::new(State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 3 6").unwrap(), exploration_param);
        for i in 0..10 {
            println!("Move: {}", i);
            mcts.run(500);
            if let Some(best_move_node) = mcts.select_best_move() {
                let best_move = unsafe { (*best_move_node).mv.clone() };
                let next_state = unsafe { (*best_move_node).state_after_move.clone() };
                mcts = MCTS::new(next_state.clone(), exploration_param);
                next_state.board.print();
                println!("Best move: {:?}", best_move.unwrap().uci());
                println!();
            }
            else{
                break;
            }
        }
    }
}