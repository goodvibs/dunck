use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;

pub fn evaluate_terminal_state(state: &State, for_color: Color) -> f64 {
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

#[derive(Debug)]
pub struct MCTSNode {
    pub state_after_move: State,
    pub mv: Option<Move>,
    visits: u32,
    value: f64,
    prior: f64,
    children: Vec<Rc<RefCell<MCTSNode>>>,
    previous_node: Option<Rc<RefCell<MCTSNode>>>,
    is_expanded: bool,
}

impl MCTSNode {
    fn new(mv: Option<Move>, previous_node: Option<Rc<RefCell<MCTSNode>>>, state_after_move: State) -> Self {
        Self {
            state_after_move,
            mv,
            visits: 0,
            value: 0.,
            prior: 0.,
            children: Vec::new(),
            previous_node,
            is_expanded: false,
        }
    }

    fn flip_values(&mut self) {
        self.value = -self.value;
        for child in &self.children {
            child.borrow_mut().flip_values();
        }
    }

    fn expand(&mut self, policy: Vec<(Move, f64)>, self_ptr: &Rc<RefCell<MCTSNode>>) {
        self.is_expanded = true;
        if policy.is_empty() {
            self.state_after_move.assume_and_update_termination();
        } else {
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
                    previous_node: Some(self_ptr.clone()),
                    is_expanded: false,
                };
                self.children.push(Rc::new(RefCell::new(new_node)));
            }
        }
    }

    fn calc_puct(&self, parent_visits: u32, c_puct: f64) -> f64 {
        let exploration = c_puct * self.prior * (parent_visits as f64).sqrt() / (1.0 + self.visits as f64);

        if self.visits == 0 {
            exploration  // Prior-driven exploration for unvisited nodes
        } else {
            let exploitation = self.value / self.visits as f64;
            exploitation + exploration
        }
    }

    fn calc_ucb1(&self, parent_visits: u32, c_ucb1: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            let exploitation = self.value / self.visits as f64;
            exploitation + c_ucb1 * (parent_visits as f64).ln() / self.visits as f64
        }
    }

    fn select_best_child(&mut self, exploration_param: f64) -> Option<Rc<RefCell<MCTSNode>>> {
        self.children.iter().max_by(|a, b| {
            let a_score = a.borrow().calc_puct(self.visits, exploration_param);
            let b_score = b.borrow().calc_puct(self.visits, exploration_param);
            a_score.partial_cmp(&b_score).unwrap()
        }).cloned()
    }

    fn backup(&mut self, value: f64) {
        self.visits += 1;
        self.value -= value;
        if let Some(previous_node) = &self.previous_node {
            previous_node.borrow_mut().backup(-1. * value);
        }
    }

    fn metadata(&self) -> String {
        format!("MCTSNode(move: {:?}, prior: {}, visits: {}, value: {})", self.mv, self.prior, self.visits, self.value)
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
        write!(f, "{}", self.fmt_helper(0, 1))
    }
}

pub struct MCTS {
    pub root: Rc<RefCell<MCTSNode>>,
    exploration_param: f64,
    evaluator: Box<dyn Evaluator>,
    save_data: bool,
    state_evaluations: Vec<(State, Evaluation)>
}

impl MCTS {
    pub fn new(state: State, exploration_param: f64, evaluator: Box<dyn Evaluator>, save_data: bool) -> Self {
        Self {
            root: Rc::new(RefCell::new(MCTSNode::new(None, None, state))),
            exploration_param,
            evaluator,
            save_data,
            state_evaluations: Vec::new()
        }
    }

    fn select_best_leaf(&self) -> Rc<RefCell<MCTSNode>> {
        let mut leaf = self.root.clone();
        loop {
            let option_best_child = leaf.borrow_mut().select_best_child(self.exploration_param);
            match option_best_child {
                Some(best_child) => {
                    leaf = best_child;
                }
                None => {
                    return leaf;
                }
            }
        }
    }

    pub fn run(&mut self, iterations: usize) {
        for _ in 0..iterations {
            let leaf = self.select_best_leaf();
            let state_after_move = leaf.borrow().state_after_move.clone();
            let evaluation = if leaf.borrow().is_expanded {
                // leaf.borrow_mut().state_after_move.assume_and_update_termination();
                let value = evaluate_terminal_state(
                    &state_after_move, state_after_move.side_to_move
                );
                Evaluation {
                    policy: Vec::with_capacity(0),
                    value,
                }
            } else {
                self.evaluator.evaluate(&state_after_move)
            };

            if self.save_data {
                self.state_evaluations.push((state_after_move, evaluation.clone()));
            }

            leaf.borrow_mut().expand(evaluation.policy, &Rc::clone(&leaf));
            leaf.borrow_mut().backup(evaluation.value);
        }
    }

    pub fn get_best_child_by_score(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow_mut().select_best_child(0.)
    }

    pub fn get_best_child_by_visits(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow_mut().children.iter().max_by(|a, b| {
            let a_score = a.borrow().visits;
            let b_score = b.borrow().visits;
            a_score.cmp(&b_score)
        }).cloned()
    }

    pub fn take_best_child(&mut self) -> Result<(State, Move), String> {
        if let Some(best_child) = self.get_best_child_by_visits() {
            let best_move = best_child.borrow().mv.clone();
            let next_state = best_child.borrow().state_after_move.clone();
            self.root = best_child;
            self.root.borrow_mut().previous_node = None;
            self.root.borrow_mut().flip_values();

            Ok((next_state, best_move.unwrap()))
        } else {
            Err("No best child found".to_string())
        }
    }

    pub fn play_game(&mut self, num_iterations_per_move: usize, max_depth: usize) -> f64 {
        let initial_side_to_move = self.root.borrow().state_after_move.side_to_move;
        for _ in 0..max_depth {
            self.run(num_iterations_per_move);
            match self.take_best_child() {
                Ok(_) => {}
                Err(_) => {
                    let final_state = self.root.borrow().state_after_move.clone();
                    assert!(final_state.termination.is_some());
                    assert!(final_state.is_unequivocally_valid());
                    return evaluate_terminal_state(&final_state, initial_side_to_move);
                }
            }
        }
        0.
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
    use crate::engine::conv_net_evaluator::ConvNetEvaluator;
    use crate::engine::material_evaluator::MaterialEvaluator;
    use crate::engine::rollout_evaluator::RolloutEvaluator;
    use super::*;

    #[test]
    fn test_mcts() {
        let exploration_param = 1.5;
        let mut mcts = MCTS::new(
            State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 5 6").unwrap(),
            // State::initial(),
            exploration_param,
            Box::new(ConvNetEvaluator::new(4, 8, true)),
            // Box::new(RolloutEvaluator::new(200)),
            // Box::new(MaterialEvaluator {}),
            true
        );
        for i in 0..10 {
            println!("Move: {}", i);
            mcts.run(400);
            println!("{}", mcts);
            let initial_state = mcts.root.borrow().state_after_move.clone();
            match mcts.take_best_child() {
                Ok((next_state, mv)) => {
                    println!("Playing best move: {:?}", mv.san(&initial_state, &next_state, &next_state.calc_legal_moves()));
                    next_state.board.print();
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    }
    
    #[test]
    fn test_play_game() {
        let exploration_param = 1.5;
        let mut mcts = MCTS::new(
            State::initial(),
            exploration_param,
            // Box::new(MaterialEvaluator {}),
            // Box::new(RolloutEvaluator::new(200)),
            Box::new(ConvNetEvaluator::new(4, 8, false)),
            true
        );
        let result = mcts.play_game(400, 300);
        for (state, evaluation) in mcts.state_evaluations.iter() {
            println!("State: {}", state.board);
            println!("Evaluation: {:?}", evaluation);
        }
        println!("Simulation result: {}", result);
    }
}