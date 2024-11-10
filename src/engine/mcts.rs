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

    fn expand(&mut self, policy: Vec<(Move, f64)>, self_ptr: &Rc<RefCell<MCTSNode>>) {
        self.is_expanded = true;
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
                    previous_node: Some(self_ptr.clone()),
                    is_expanded: false,
                };
                self.children.push(Rc::new(RefCell::new(new_node)));
            }
        }
    }

    fn calc_ucb1(&self, parent_visits: u32, c_puct: f64) -> f64 {
        let exploration = c_puct * self.prior * (parent_visits as f64).sqrt() / (1.0 + self.visits as f64);

        if self.visits == 0 {
            exploration  // Prior-driven exploration for unvisited nodes
        } else {
            let exploitation = self.value / self.visits as f64;
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
        write!(f, "{}", self.fmt_helper(0, 2))
    }
}

pub struct MCTS {
    root: Rc<RefCell<MCTSNode>>,
    exploration_param: f64,
    evaluator: Box<dyn Evaluator>
}

impl MCTS {
    pub fn new(state: State, exploration_param: f64, evaluator: Box<dyn Evaluator>) -> Self {
        Self {
            root: Rc::new(RefCell::new(MCTSNode::new(None, None, state))),
            exploration_param,
            evaluator
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

    pub fn run(&mut self, iterations: u32) {
        for _ in 0..iterations {
            let leaf = self.select_best_leaf();
            let evaluation = if leaf.borrow().is_expanded {
                let value = evaluate_terminal_state(
                    &leaf.borrow().state_after_move, leaf.borrow().state_after_move.side_to_move
                );
                Evaluation {
                    policy: Vec::with_capacity(0),
                    value,
                }
            } else { 
                self.evaluator.evaluate(&leaf.borrow().state_after_move)
            };
            let leaf_ptr = Rc::clone(&leaf);
            leaf.borrow_mut().expand(evaluation.policy, &leaf_ptr);
            leaf.borrow_mut().backup(evaluation.value);
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
    use crate::engine::material_evaluator::MaterialEvaluator;
    use crate::engine::rollout_evaluator::RolloutEvaluator;
    use super::*;

    #[test]
    fn test_mcts() {
        let exploration_param = 1.5;
        let evaluator = Box::new(RolloutEvaluator::new(200));
        // let evaluator = Box::new(MaterialEvaluator {});
        let mut mcts = MCTS::new(
            State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 5 6").unwrap(),
            // State::initial(),
            exploration_param,
            evaluator.clone()
        );
        for i in 0..1 {
            println!("Move: {}", i);
            mcts.run(800);
            println!("{}", mcts);
            if let Some(best_move_node) = mcts.select_best_move() {
                let best_move = best_move_node.borrow().mv.clone();
                let next_state = best_move_node.borrow().state_after_move.clone();
                mcts = MCTS::new(
                    next_state.clone(),
                    exploration_param,
                    evaluator.clone()
                );
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