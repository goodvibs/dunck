use lazy_static::lazy_static;
use tch::{nn, nn::Module, nn::OptimizerConfig, Tensor, Device, Kind};
use crate::r#move::Move;
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, PieceType};

lazy_static! {
    static ref DEVICE: Device = Device::cuda_if_available();
    static ref VS: nn::VarStore = nn::VarStore::new(*DEVICE);
    static ref MODEL: ChessModel = ChessModel::new(&VS.root());
}

// Constants for the input tensor
const NUM_PIECE_TYPE_BITS: i64 = 6; // 6 piece types
const NUM_COLOR_BITS: i64 = 2; // 2 colors
const NUM_BITS_PER_BOARD: i64 = NUM_PIECE_TYPE_BITS * NUM_COLOR_BITS;

const NUM_STATES_LOOKBACK: i64 = 0; // no lookback
const NUM_STATES_TO_CONSIDER: i64 = NUM_STATES_LOOKBACK + 1;

const NUM_BOARD_BITS: i64 = NUM_BITS_PER_BOARD * NUM_STATES_TO_CONSIDER; // 12 bits for board(s)

const NUM_CASTLING_BITS: i64 = 4; // 4 castling rights
const NUM_SIDE_TO_MOVE_BITS: i64 = 1; // 1 bit for side to move
const NUM_METADATA_BITS: i64 = NUM_CASTLING_BITS + NUM_SIDE_TO_MOVE_BITS; // 5 bits for metadata

const NUM_POSITION_BITS: i64 = NUM_BOARD_BITS + NUM_METADATA_BITS; // Number of 8x8 planes in the input tensor

const NUM_TARGET_SQUARE_POSSIBILITIES: i64 = 73; // Number of possible target squares for a move
const NUM_OUTPUT_POLICY_MOVES: i64 = 8 * 8 * NUM_TARGET_SQUARE_POSSIBILITIES; // 4672 possible moves for policy head
const NUM_INITIAL_CONV_OUTPUT_CHANNELS: i64 = 32; // Output channels for initial conv layer


pub fn state_to_tensor(state: &State) -> Tensor {
    // Initialize a tensor with shape [1, 17, 8, 8], where:
    // - 1 is the batch size
    // - 17 is the number of channels
    // - 8x8 is the board size
    let tensor = Tensor::zeros(&[1, NUM_POSITION_BITS, 8, 8], (Kind::Float, *DEVICE));

    // Channel 0-5: White pieces (one channel per piece type)
    // Channel 6-11: Black pieces (one channel per piece type)
    for (i, piece_type) in PieceType::iter_pieces().enumerate() {
        // White pieces
        let white_mask = state.board.piece_type_masks[piece_type as usize] & state.board.color_masks[Color::White as usize];
        for square in get_squares_from_mask_iter(white_mask) {
            let _ = tensor.get(0).get(i as i64).get(square.get_rank() as i64).get(square.get_file() as i64).fill_(1.);
        }

        // Black pieces
        let black_mask = state.board.piece_type_masks[piece_type as usize] & state.board.color_masks[Color::Black as usize];
        for square in get_squares_from_mask_iter(black_mask) {
            let _ = tensor.get(0).get((i + 6) as i64).get(square.get_rank() as i64).get(square.get_file() as i64).fill_(1.);
        }
    }

    // Channel 12: Side to move (1 if white to move, 0 if black to move)
    let _ = tensor.get(0).get(12).fill_(
        if state.side_to_move == Color::White { 1. } else { 0. }
    );
    // Channel 13-16: Castling rights
    let castling_rights = state.context.borrow().castling_rights;
    let _ = tensor.get(0).get(13).fill_(
        if castling_rights & 0b1000 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(0).get(14).fill_(
        if castling_rights & 0b0100 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(0).get(15).fill_(
        if castling_rights & 0b0010 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(0).get(16).fill_(
        if castling_rights & 0b0001 != 0 { 1. } else { 0. }
    );

    tensor
}


pub fn get_move_mask(moves: &Vec<Move>) -> Tensor {
    // Initialize a mask tensor with shape [8, 8, 73] (8x8 board, 73 possible moves)
    let mut mask = Tensor::zeros(&[8, 8, NUM_TARGET_SQUARE_POSSIBILITIES], (Kind::Float, *DEVICE));

    // Set the mask to 1 for each legal move
    for mv in moves {
        let source_square = mv.get_source();
        let destination_square = mv.get_destination();
        
        let src_rank = source_square.get_rank() as i64;
        let src_file = source_square.get_file() as i64;
        let dst_rank = destination_square.get_rank() as i64;
        let dst_file = destination_square.get_file() as i64;
        
        let target_square_index = dst_rank * 8 + dst_file;
        let _ = mask.get(src_rank).get(src_file).get(target_square_index).fill_(1.);
    }

    mask
}


pub fn renormalize_policy(policy_output: Tensor, legal_move_mask: Tensor) -> Tensor {
    // Apply the mask to zero out illegal moves
    let masked_policy = policy_output * &legal_move_mask;

    // Sum the masked probabilities to get the total probability of legal moves
    let sum_legal_probs_tensor = masked_policy.sum(Kind::Float);
    let sum_legal_probs = sum_legal_probs_tensor.double_value(&[]);

    // Avoid division by zero in case all moves are illegal
    if sum_legal_probs > 0. {
        // Renormalize the masked probabilities by dividing by the total sum
        masked_policy / sum_legal_probs
    } else {
        // If there are no legal moves, return the mask itself as probabilities (all zero)
        legal_move_mask
    }
}

// Define a Residual Block
#[derive(Debug)]
struct ResidualBlock {
    conv1: nn::Conv2D,
    conv2: nn::Conv2D,
}

impl ResidualBlock {
    fn new(vs: &nn::Path, channels: i64) -> ResidualBlock {
        // Initialize two convolutional layers with ReLU activations
        let conv1 = nn::conv2d(vs, channels, channels, 3, nn::ConvConfig { padding: 1, ..Default::default() });
        let conv2 = nn::conv2d(vs, channels, channels, 3, nn::ConvConfig { padding: 1, ..Default::default() });

        ResidualBlock { conv1, conv2 }
    }

    // Forward pass for the residual block
    fn forward(&self, x: &Tensor) -> Tensor {
        let residual = x; // Save input for skip connection
        let x = x.relu().apply(&self.conv1);
        let x = x.relu().apply(&self.conv2);
        x + residual // Add skip connection
    }
}

// Define the main model structure
#[derive(Debug)]
struct ChessModel {
    conv1: nn::Conv2D,
    residual_block1: ResidualBlock,
    residual_block2: ResidualBlock,
    fc_policy: nn::Linear,
    fc_value: nn::Linear,
}

impl ChessModel {
    fn new(vs: &nn::Path) -> ChessModel {
        // Initial convolutional layer
        let conv1 = nn::conv2d(vs, NUM_POSITION_BITS, NUM_INITIAL_CONV_OUTPUT_CHANNELS, 3, nn::ConvConfig { padding: 1, ..Default::default() }); // 17 input channels, 32 output channels

        // Two residual blocks with 32 channels each
        let residual_block1 = ResidualBlock::new(&vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS);
        let residual_block2 = ResidualBlock::new(&vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS);

        // Fully connected layers for policy and value heads
        let fc_policy = nn::linear(
            vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS * 8 * 8, NUM_OUTPUT_POLICY_MOVES, Default::default()
        ); // Map to 4672 possible moves
        let fc_value = nn::linear(
            vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS * 8 * 8, 1, Default::default()
        ); // Map to a single scalar value

        ChessModel {
            conv1,
            residual_block1,
            residual_block2,
            fc_policy,
            fc_value,
        }
    }

    // Forward pass
    fn forward(&self, x: &Tensor) -> (Tensor, Tensor) {
        // Apply the initial convolutional layer with ReLU activation
        let x = x.view([-1, NUM_POSITION_BITS, 8, 8]).apply(&self.conv1).relu();

        // Pass through residual blocks
        let x = self.residual_block1.forward(&x);
        let x = self.residual_block2.forward(&x);

        // Flatten for fully connected layers
        let x = x.view([-1, NUM_INITIAL_CONV_OUTPUT_CHANNELS * 8 * 8]);

        // Policy head: Softmax over 4672 possible moves
        let policy = self.fc_policy.forward(&x).view(
            [-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES]
        ).softmax(-1, Kind::Float); // Softmax for move probabilities

        // Value head: Tanh for output between -1 and 1
        let value = self.fc_value.forward(&x).tanh();

        (policy, value)
    }
}

unsafe impl Sync for ChessModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_to_tensor() {
        let state = State::initial();
        let tensor = state_to_tensor(&state);

        assert_eq!(tensor.size(), [1, NUM_POSITION_BITS, 8, 8]);
    }

    #[test]
    fn test_chess_model() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ChessModel::new(&vs.root());

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor);

        assert_eq!(policy.size(), [1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES]);
        assert_eq!(value.size(), [1, 1]);
    }

    #[test]
    fn test_training() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ChessModel::new(&vs.root());

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor);

        let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES], (Kind::Float, *DEVICE));
        let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

        let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
        let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);

        let loss = policy_loss + value_loss;

        let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();
        optimizer.backward_step(&loss);
    }

    #[test]
    fn test_train_100_iterations() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ChessModel::new(&vs.root());

        for _ in 0..100 {
            let input_tensor = state_to_tensor(&State::initial());
            let (policy, value) = model.forward(&input_tensor);

            let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES], (Kind::Float, *DEVICE));
            let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

            let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
            let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);

            let loss = policy_loss + value_loss;

            let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();
            optimizer.backward_step(&loss);
        }
    }
}