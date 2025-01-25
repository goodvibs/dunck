use std::cell::RefCell;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use rand::distributions::Distribution;
use rand_distr::Gamma;
use crate::engine::evaluation::{get_value_at_terminal_state, Evaluation, Evaluator};
use crate::engine::mcts::mcts_node::MCTSNode;
use crate::r#move::Move;
use crate::state::{State};

// fn generate_dirichlet_noise(num_moves: usize, alpha: f64) -> Vec<f64> {
//     let gamma = Gamma::new(alpha, 1.0).expect("Invalid alpha for Dirichlet");
//     let mut rng = rand::thread_rng();
//     let mut noise: Vec<f64> = (0..num_moves).map(|_| gamma.sample(&mut rng)).collect();
// 
//     // Normalize the noise to sum to 1
//     let sum: f64 = noise.iter().sum();
//     noise.iter_mut().for_each(|n| *n /= sum);
//     noise
// }

pub fn calc_uct_score(node: &MCTSNode, parent_visits: u32, exploration_constant: f64) -> f64 {
    if node.visits == 0 {
        f64::INFINITY
    } else {
        let exploitation = node.value / node.visits as f64;
        let exploration = exploration_constant * ((parent_visits as f64).ln() / node.visits as f64).sqrt();
        exploitation + exploration
    }
}

pub fn calc_puct_score(node: &MCTSNode, parent_visits: u32, exploration_constant: f64) -> f64 {
    let exploration = exploration_constant * node.prior * (parent_visits as f64).sqrt() / (1.0 + node.visits as f64);

    if node.visits == 0 {
        exploration  // Prior-driven exploration for unvisited nodes
    } else {
        let exploitation = node.value / node.visits as f64;
        exploitation + exploration
    }
}

pub struct MCTS<'a> {
    pub root: Rc<RefCell<MCTSNode>>,
    pub exploration_param: f64,
    pub evaluator: &'a dyn Evaluator,
    pub calc_node_score: &'static dyn Fn(&MCTSNode, u32, f64) -> f64,
    pub save_data: bool,
    pub state_evaluations: Vec<(State, Evaluation)>
}

impl<'a> MCTS<'a> {
    pub fn new(
        state: State,
        exploration_param: f64,
        evaluator: &'a dyn Evaluator,
        calc_node_score: &'static dyn Fn(&MCTSNode, u32, f64) -> f64,
        save_data: bool
    ) -> Self {
        Self {
            root: Rc::new(RefCell::new(MCTSNode::new(None, None, state))),
            exploration_param,
            evaluator,
            calc_node_score,
            save_data,
            state_evaluations: Vec::new()
        }
    }

    fn select_best_leaf(&self) -> Rc<RefCell<MCTSNode>> {
        let mut leaf = self.root.clone();
        loop {
            let option_best_child = leaf.borrow_mut().select_best_child(self.calc_node_score, self.exploration_param);
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
                let value = get_value_at_terminal_state(
                    &state_after_move, state_after_move.side_to_move
                );
                Evaluation {
                    policy: Vec::with_capacity(0),
                    value,
                }
            } else {
                self.evaluator.evaluate(&state_after_move)
            };

            // // Apply Dirichlet noise at the root node
            // if Rc::ptr_eq(&self.root, &leaf) {
            //     let alpha = 0.3;
            //     let epsilon = 0.25;
            //     let num_moves = evaluation.policy.len();
            // 
            //     if num_moves > 0 {
            //         let noise = generate_dirichlet_noise(num_moves, alpha);
            // 
            //         for (i, (_, prob)) in evaluation.policy.iter_mut().enumerate() {
            //             *prob = (1.0 - epsilon) * *prob + epsilon * noise[i];
            //         }
            //     }
            // }


            if self.save_data {
                self.state_evaluations.push((state_after_move, evaluation.clone()));
            }

            leaf.borrow_mut().expand(evaluation.policy, &Rc::clone(&leaf));
            leaf.borrow_mut().backup(evaluation.value);
        }
    }

    pub fn get_best_child_by_score(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow_mut().select_best_child(self.calc_node_score, 0.)
    }

    pub fn get_best_child_by_visits(&self) -> Option<Rc<RefCell<MCTSNode>>> {
        self.root.borrow_mut().children.iter().max_by(|a, b| {
            let a_score = a.borrow().visits;
            let b_score = b.borrow().visits;
            a_score.cmp(&b_score)
        }).cloned()
    }
    
    pub fn take_child_with_move(&mut self, mv: Move, expand_if_unexpanded: bool) -> Result<(), String> {
        if !self.root.borrow().is_expanded {
            if expand_if_unexpanded {
                let evaluation = self.evaluator.evaluate(&self.root.borrow().state_after_move);
                self.root.borrow_mut().expand(evaluation.policy, &self.root);
            } else {
                return Err("Root node is not expanded".to_string());
            }
        }
        
        let mut new_root = None;
        {
            let root = self.root.borrow();
            let children_iter = root.children.iter();
            for child in children_iter {
                if child.borrow().mv == Some(mv) {
                    new_root = Some(Rc::clone(child));
                    break;
                }
            }
        }
        if let Some(new_root) = new_root {
            self.root = new_root;
            self.root.borrow_mut().previous_node = None;
            self.root.borrow_mut().flip_values();
            Ok(())
        } else {
            Err("No child found".to_string())
        }
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
                    return get_value_at_terminal_state(&final_state, initial_side_to_move);
                }
            }
        }
        0.
    }
}

impl<'a> Display for MCTS<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root.borrow())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::evaluators::neural::conv_net_evaluator::ConvNetEvaluator;
    use crate::engine::evaluators::random_rollout::RolloutEvaluator;
    use super::*;

    #[test]
    fn test_mcts() {
        // let evaluator = ConvNetEvaluator::new(4, 8, true);
        let evaluator = RolloutEvaluator::new(300);
        let exploration_param = 1.5;
        let mut mcts = MCTS::new(
            State::from_fen("r1n1k3/p2p1pbr/B1p1pnp1/2qPN3/4P3/R1N1BQ1P/1PP2P1P/4K2R w Kq - 5 6").unwrap(),
            // State::initial(),
            exploration_param,
            &evaluator,
            &calc_uct_score,
            true
        );
        for i in 0..1 {
            println!("Move: {}", i);
            mcts.run(1000);
            println!("{}", mcts);
            let initial_state = mcts.root.borrow().state_after_move.clone();
            match mcts.take_best_child() {
                Ok((next_state, mv)) => {
                    println!("Playing best move: {:?}", mv.to_san(&initial_state, &next_state, &next_state.calc_legal_moves()));
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
        let evaluator = ConvNetEvaluator::new(4, 8);
        let exploration_param = 1.5;
        let mut mcts = MCTS::new(
            State::initial(),
            exploration_param,
            &evaluator,
            &calc_uct_score,
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