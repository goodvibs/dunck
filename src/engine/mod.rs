use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;
use fastrand;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

fn ucb1(node_value: f64, parent_visits: u32, child_visits: u32, exploration_param: f64) -> f64 {
    node_value / child_visits as f64
        + exploration_param * ((parent_visits as f64).ln() / child_visits as f64).sqrt()
}

fn simulate_rollout(mut state: State) -> f64 {
    let current_side_to_move = state.side_to_move;
    let mut rng = fastrand::Rng::new();
    loop {
        let moves = state.calc_legal_moves();
        if moves.is_empty() {
            return evaluate_game_state(&state, current_side_to_move);
        } else {
            let rand_idx = rng.usize(..moves.len());
            let mv = moves[rand_idx];
            state.make_move(mv);
        }
    }
}

fn evaluate_game_state(state: &State, for_color: Color) -> f64 {
    match state.termination {
        None | Some(Termination::Checkmate) => {
            let checkmated_side = state.side_to_move;
            if checkmated_side == for_color {
                0.0
            } else {
                1.0
            }
        }
        Some(_) => 0.5,
    }
}

struct MCTSNode {
    state: State,
    visits: u32,
    value: f64,
    children: Vec<Rc<RefCell<MCTSNode>>>,
    parent: Option<Weak<RefCell<MCTSNode>>>,
}

impl MCTSNode {
    fn new(state: State) -> Self {
        Self {
            state,
            visits: 0,
            value: 0.0,
            children: Vec::new(),
            parent: None,
        }
    }

    fn run(&mut self, exploration_param: f64, self_as_parent: Weak<RefCell<MCTSNode>>) {
        let possible_selected_node = self.select_child_with_ucb1(exploration_param);

        match possible_selected_node {
            Some(selected_node) => {
                // Upgrade the weak reference temporarily for recursive call
                selected_node.borrow_mut().run(exploration_param, Rc::downgrade(&selected_node));
            }
            None => {
                if self.visits == 0 {
                    let value = simulate_rollout(self.state.clone());
                    self.backpropagate(value);
                } else {
                    let legal_moves = self.state.calc_legal_moves();
                    for legal_move in legal_moves {
                        let mut new_state = self.state.clone();
                        new_state.make_move(legal_move);
                        let mut new_node = MCTSNode::new(new_state);
                        new_node.parent = Some(self_as_parent.clone()); // Pass weak reference
                        self.children.push(Rc::new(RefCell::new(new_node)));
                    }
                    if !self.children.is_empty() {
                        let random_child = self.children[fastrand::usize(..self.children.len())].clone();
                        random_child.borrow_mut().run(exploration_param, Rc::downgrade(&random_child));
                    }
                }
            }
        }
    }

    fn select_child_with_ucb1(&mut self, exploration_param: f64) -> Option<Rc<RefCell<MCTSNode>>> {
        self.children.iter()
            .max_by(|a, b| {
                ucb1(a.borrow().value, self.visits, a.borrow().visits, exploration_param)
                    .partial_cmp(&ucb1(b.borrow().value, self.visits, b.borrow().visits, exploration_param))
                    .unwrap()
            })
            .cloned() // Return an owned Rc reference
    }

    fn backpropagate(&mut self, value: f64) {
        self.visits += 1;
        self.value += value;

        // Clone the parent reference to extend its lifetime beyond the mutable borrow
        let parent_option = self.parent.clone();

        // Mutable borrow ends here

        if let Some(weak_parent) = parent_option {
            if let Some(parent_rc) = weak_parent.upgrade() {
                parent_rc.borrow_mut().backpropagate(value);
            }
        }
    }
}

struct MCTS {
    root: Rc<RefCell<MCTSNode>>,
    exploration_param: f64,
}

impl MCTS {
    fn new(state: State, exploration_param: f64) -> Self {
        Self {
            root: Rc::new(RefCell::new(MCTSNode::new(state))),
            exploration_param,
        }
    }

    fn run(&mut self, iterations: u32) {
        for _ in 0..iterations {
            self.root.borrow_mut().run(self.exploration_param, Rc::downgrade(&self.root));
        }
    }

    fn select_best_move(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow().children.iter().max_by_key(|child| child.borrow().visits).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcts() {
        let mut mcts = MCTS::new(State::initial(), 1.41);
        mcts.run(1000);
        if let Some(best_move) = mcts.select_best_move() {
            best_move.borrow().state.board.print();
        }
    }
}