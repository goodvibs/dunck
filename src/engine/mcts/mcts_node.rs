use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use crate::r#move::Move;
use crate::state::State;

#[derive(Debug)]
pub struct MCTSNode {
    pub state_after_move: State,
    pub mv: Option<Move>,
    pub visits: u32,
    pub value: f64,
    pub prior: f64,
    pub children: Vec<Rc<RefCell<MCTSNode>>>,
    pub previous_node: Option<Rc<RefCell<MCTSNode>>>,
    pub is_expanded: bool,
}

impl MCTSNode {
    pub fn new(mv: Option<Move>, previous_node: Option<Rc<RefCell<MCTSNode>>>, state_after_move: State) -> Self {
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

    pub fn flip_values(&mut self) {
        self.value = -self.value;
        for child in &self.children {
            child.borrow_mut().flip_values();
        }
    }

    pub fn expand(&mut self, policy: Vec<(Move, f64)>, self_ptr: &Rc<RefCell<MCTSNode>>) {
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

    pub fn select_best_child(&mut self, calc_score: &'static dyn Fn(&MCTSNode, u32, f64) -> f64,  exploration_param: f64) -> Option<Rc<RefCell<MCTSNode>>> {
        self.children.iter().max_by(|a, b| {
            let a_score = calc_score(&*a.borrow(), self.visits, exploration_param);
            let b_score = calc_score(&*b.borrow(), self.visits, exploration_param);
            a_score.partial_cmp(&b_score).unwrap()
        }).cloned()
    }

    pub fn backup(&mut self, value: f64) {
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