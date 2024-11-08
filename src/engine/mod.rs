mod neural_network;

use std::cell::RefCell;
use std::cmp::max_by;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use rand::prelude::SliceRandom;
use rand::Rng;
use crate::r#move::r#move;
use crate::state::{Context, State, Termination};
use crate::utils::{Color, PieceType};

pub struct Evaluation {
    pub policy: Vec<(Move, f64)>,
    pub value: f64,
}

fn evaluate_state(state: &State, for_color: Color) -> Evaluation {
    let mut state = state.clone();
    let legal_moves = state.calc_legal_moves();

    let mut policy = Vec::with_capacity(legal_moves.len());
    let mut rng = rand::thread_rng();
    for mv in legal_moves {
        state.make_move(mv);
        let prior = evaluate_non_terminal_state(&state, for_color.flip()) + rng.gen_range(0.0..0.0001);
        policy.push((mv, prior));
        state.unmake_move(mv);
    }

    let value = if policy.is_empty() {
        state.assume_and_update_termination();
        evaluate_terminal_state(&state, for_color)
    } else {
        let max = policy.iter().map(|(_, prior)| *prior).fold(f64::NEG_INFINITY, f64::max);
        1. - max
    };

    Evaluation {
        policy,
        value,
    }
}

fn simulate_rollout(mut state: State, for_color: Color) -> f64 {
    const MAX_ROLLOUT_DEPTH: u32 = 50;
    // let mut rng = fastrand::Rng::new();
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
                0.
            } else {
                1.
            }
        }
        _ => 0.5
    }
}

pub fn evaluate_non_terminal_state(state: &State, for_color: Color) -> f64 {
    let mut scores = [0.0, 0.0];
    for color in Color::iter() {
        let color_mask = state.board.color_masks[color as usize];
        for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::Queen) {
            let piece_mask = state.board.piece_type_masks[piece_type as usize];
            let mask = color_mask & piece_mask;
            let count = mask.count_ones() as f64;
            scores[color as usize] += PIECE_VALUES[piece_type as usize - 1] * count;
        }
    }

    // Calculate score difference from perspective of for_color
    let score_diff = scores[for_color as usize] - scores[for_color.flip() as usize];

    sigmoid(score_diff, 0.5) // Normalize to [0, 1]
}

fn sigmoid(x: f64, a: f64) -> f64 {
    1.0 / (1.0 + (-a * x).exp())
}

const PIECE_VALUES: [f64; 5] = [
    1.0,  // Pawn
    3.0,  // Knight
    3.0,  // Bishop
    5.0,  // Rook
    9.0   // Queen
];

#[derive(Debug)]
pub struct MCTSNode {
    pub state_after_move: State,
    pub mv: Option<Move>,
    visits: u32,
    value: f64,
    prior: f64,
    children: Vec<Rc<RefCell<MCTSNode>>>,
}

impl MCTSNode {
    fn new(mv: Option<Move>, state_after_move: State) -> Self {
        Self {
            state_after_move,
            mv,
            visits: 0,
            value: 0.0,
            prior: 0.0,
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
                // value = simulate_rollout(self.state_after_move.clone(), self.state_after_move.side_to_move.flip());
                let evaluation = evaluate_state(&self.state_after_move, self.state_after_move.side_to_move.flip());
                self.expand(evaluation.policy);
                value = evaluation.value;
            }
        }
        self.visits += 1;
        self.value += value;
        // println!("{}", self.metadata());
        value
    }

    fn expand(&mut self, policy: Vec<(Move, f64)>) {
        if !policy.is_empty() {
            for (legal_move, prior) in policy {
                let mut new_state = self.state_after_move.clone();
                new_state.make_move(legal_move);
                let new_node = MCTSNode {
                    state_after_move: new_state,
                    mv: Some(legal_move),
                    visits: 0,
                    value: 0.0,
                    prior,
                    children: Vec::new(),
                };
                self.children.push(Rc::new(RefCell::new(new_node)));
            }
        }
    }

    fn calc_ucb1(&self, parent_visits: u32, exploration_param: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            let exploitation = self.value / self.visits as f64;
            let exploration = exploration_param * ((parent_visits as f64).ln() / self.visits as f64).sqrt() * self.prior;
            exploitation + exploration
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
        let mut mcts = MCTS::new(State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 5 6").unwrap(), exploration_param);
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