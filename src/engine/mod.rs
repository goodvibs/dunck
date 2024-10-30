mod evaluate;

use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::evaluate::evaluate_non_terminal_state;
use crate::r#move::Move;
use crate::state::{Context, State, Termination};
use crate::utils::Color;

const MAX_ROLLOUT_DEPTH: u32 = 50;

fn simulate_rollout(mut state: State) -> f64 {
    let current_side_to_move = state.side_to_move;
    let mut rng = fastrand::Rng::new();
    let mut i = 0;
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            return evaluate_terminal_state(&state, current_side_to_move);
        } else {
            let rand_idx = rng.usize(..moves.len());
            let mv = moves[rand_idx];
            state.make_move(mv);
        }
        i += 1;
        if i >= MAX_ROLLOUT_DEPTH {
            return evaluate_non_terminal_state(&state, current_side_to_move);
        }
    }
}

fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
    // state.board.print();
    // println!("{:?}", state.termination);
    // println!("{:?}", state.side_to_move);
    // println!("{:?}", state.halfmove);
    // println!("{:?}", state.context.borrow().halfmove_clock);
    // println!();
    let termination = match &state.termination {
        Some(termination) => termination,
        None => &match state.board.is_color_in_check(state.side_to_move) {
            true => Termination::Checkmate,
            false => Termination::Stalemate,
        }
    };
    // println!("{:?}", termination);
    match termination {
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

#[derive(Debug)]
struct MCTSNode {
    state: State,
    visits: u32,
    value: f64,
    children: Vec<*mut MCTSNode>,
}

impl MCTSNode {
    fn new(state: State) -> Self {
        Self {
            state,
            visits: 0,
            value: 0.0,
            children: Vec::new(),
        }
    }

    fn run(&mut self, exploration_param: f64) -> f64 {
        let possible_selected_child = self.select_child_with_ucb1(exploration_param);
        let value;

        match possible_selected_child {
            Some(selected_child) => unsafe { // has at least one child
                value = (*selected_child).run(exploration_param);
            }
            None => unsafe { // no children
                if self.visits == 0 {
                    value = simulate_rollout(self.state.clone());
                } else {
                    let legal_moves = self.state.calc_legal_moves();
                    for legal_move in legal_moves {
                        let mut new_state = self.state.clone();
                        new_state.make_move(legal_move);
                        let new_node = MCTSNode::new(new_state);
                        self.children.push(Box::into_raw(Box::new(new_node)));
                    }
                    if !self.children.is_empty() {
                        let random_child = self.children[0].clone();
                        value = (*random_child).run(exploration_param);
                    }
                    else { // terminal state
                        value = evaluate_terminal_state(&self.state, self.state.side_to_move);
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
            root: Box::into_raw(Box::new(MCTSNode::new(state))),
            exploration_param,
        }
    }

    fn run(&mut self, iterations: u32) {
        for _ in 0..iterations {
            unsafe { (*self.root).run(self.exploration_param) };
        }
    }

    fn select_best_move(&self) -> Option<*mut MCTSNode> {
        unsafe { (*self.root).children.iter().max_by_key(|child| (***child).visits).cloned() }
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

    fn run_simulations(state: State, iterations: u32, exploration_param: f64) -> MCTSNode {
        let mut root = MCTSNode::new(state);
        for _ in 0..iterations {
            root.run(exploration_param);
        }
        root
    }

    #[test]
    fn test_mcts() {
        let exploration_param = 1.41;
        let mut mcts = MCTS::new(State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 3 6").unwrap(), exploration_param);
        for i in 0..10 {
            println!("Move: {}", i);
            mcts.run(1000);
            if let Some(best_move) = mcts.select_best_move() {
                let next_state = unsafe { (*best_move).state.clone() };
                mcts = MCTS::new(next_state.clone(), exploration_param);
                next_state.board.print();
            }
            else{
                break;
            }
        }
    }
}