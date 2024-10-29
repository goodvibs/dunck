use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;
use fastrand;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

fn ucb1(node_value: f64, parent_visits: u32, child_visits: u32, exploration_param: f64) -> f64 {
    if child_visits == 0 {
        f64::INFINITY
    } else {
        node_value / child_visits as f64
            + exploration_param * ((parent_visits as f64).ln() / child_visits as f64).sqrt()
    }
}

fn simulate_rollout(mut state: State) -> f64 {
    let current_side_to_move = state.side_to_move;
    let mut rng = fastrand::Rng::new();
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            return evaluate_terminal_state(&state, current_side_to_move);
        } else {
            let rand_idx = rng.usize(..moves.len());
            let mv = moves[rand_idx];
            state.make_move(mv);
        }
    }
}

fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
    state.board.print();
    println!("{:?}", state.termination);
    println!("{:?}", state.side_to_move);
    println!("{:?}", state.halfmove);
    println!("{:?}", state.context.borrow().halfmove_clock);
    println!();
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
            Some(selected_child) => unsafe {
                value = (*selected_child).run(exploration_param);
            }
            None => unsafe {
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

    fn select_child_with_ucb1(&mut self, exploration_param: f64) -> Option<*mut MCTSNode> {
        unsafe {
            self.children.iter().max_by(|a, b| {
                let a_ucb1 = ucb1((***a).value, self.visits, (***a).visits, exploration_param);
                let b_ucb1 = ucb1((***b).value, self.visits, (***b).visits, exploration_param);
                a_ucb1.partial_cmp(&b_ucb1).expect("Failed to compare UCB1 values")
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
    use super::*;

    #[test]
    fn test_mcts() {
        let mut mcts = MCTS::new(State::initial(), 1.41);
        for i in 0..10 {
            println!("Iteration: {}", i);
            mcts.run(100);
            if let Some(best_move) = mcts.select_best_move() {
                let next_state = unsafe { (*best_move).state.clone() };
                mcts = MCTS::new(next_state.clone(), 1.41);
                next_state.board.print();
            }
            else{
                break;
            }
        }
    }
}