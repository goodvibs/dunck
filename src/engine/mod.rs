mod evaluate;

use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::engine::evaluate::evaluate_non_terminal_state;
use crate::r#move::Move;
use crate::state::{Context, State, Termination};
use crate::utils::Color;

const MAX_ROLLOUT_DEPTH: u32 = 500;

fn simulate_rollout(mut state: State) -> f64 {
    let for_color = state.side_to_move;
    let mut rng = fastrand::Rng::new();
    let mut i = 0;
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            state.assume_and_update_termination();
            return evaluate_terminal_state(&state, for_color);
        } else {
            let rand_idx = rng.usize(..moves.len());
            let mv = moves[rand_idx];
            state.make_move(mv);
        }
        i += 1;
        if i >= MAX_ROLLOUT_DEPTH {
            return evaluate_non_terminal_state(&state, for_color);
        }
    }
}

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

#[derive(Debug)]
pub struct MCTSNode {
    pub state_after_move: State,
    pub mv: Option<Move>,
    visits: u32,
    value: f64,
    children: Vec<Rc<RefCell<MCTSNode>>>,
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
        let possible_selected_child = self.select_best_child(exploration_param);
        let value;

        match possible_selected_child {
            Some(best_child) => {
                value = 1. - best_child.borrow_mut().run(exploration_param);
            }
            None => { // self is a leaf node
                value = simulate_rollout(self.state_after_move.clone());
                self.expand();
            }
        }
        self.visits += 1;
        self.value += value;
        // println!("{}", self.metadata());
        value
    }

    fn expand(&mut self) {
        let legal_moves = self.state_after_move.calc_legal_moves();
        if legal_moves.is_empty() {
            if self.state_after_move.termination.is_none() {
                self.state_after_move.assume_and_update_termination();
            }
        }
        else {
            for legal_move in legal_moves {
                let mut new_state = self.state_after_move.clone();
                new_state.make_move(legal_move);
                let new_node = MCTSNode::new(Some(legal_move), new_state);
                self.children.push(Rc::new(RefCell::new(new_node)));
            }
        }
    }

    fn calc_ucb1(&self, parent_visits: u32, exploration_param: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            self.value / self.visits as f64
                + exploration_param * ((parent_visits as f64).ln() / self.visits as f64).sqrt()
        }
    }

    fn select_best_child(&mut self, exploration_param: f64) -> Option<Rc<RefCell<MCTSNode>>> {
        self.children.iter().max_by(|a, b| {
            let a_score = a.borrow().calc_ucb1(self.visits, exploration_param);
            let b_score = b.borrow().calc_ucb1(self.visits, exploration_param);
            a_score.partial_cmp(&b_score).unwrap()
        }).cloned()
    }

    fn metadata(&self) -> String {
        format!("MCTSNode(move: {:?}, visits: {}, value: {})", self.mv, self.visits, self.value)
    }

    fn fmt_helper(&self, depth: usize, depth_limit: usize) -> String {
        let mut s = format!("{}{}\n", "| ".repeat(depth), self.metadata());
        if depth < depth_limit {
            for child in &self.children {
                s += &child.borrow().fmt_helper(depth + 1, depth_limit);
            }
        }
        s
    }
}

impl Display for MCTSNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt_helper(0, 2))
    }
}

pub struct MCTS {
    root: Rc<RefCell<MCTSNode>>,
    exploration_param: f64,
}

impl MCTS {
    pub fn new(state: State, exploration_param: f64) -> Self {
        Self {
            root: Rc::new(RefCell::new(MCTSNode::new(None, state))),
            exploration_param,
        }
    }

    pub fn run(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.root.borrow_mut().run(self.exploration_param);
        }
    }

    pub fn select_best_move(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow_mut().select_best_child(0.)
    }
}

impl Display for MCTS {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root.borrow())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn test_mcts() {
        let exploration_param = 2.;
        let mut mcts = MCTS::new(State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 3 6").unwrap(), exploration_param);
        for i in 0..1 {
            println!("Move: {}", i);
            mcts.run(10000);
            println!("{}", mcts);
            if let Some(best_move_node) = mcts.select_best_move() {
                let best_move = best_move_node.borrow().mv.clone();
                let next_state = best_move_node.borrow().state_after_move.clone();
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