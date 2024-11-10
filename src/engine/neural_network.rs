use lazy_static::lazy_static;
use tch::{nn, nn::Module, nn::OptimizerConfig, Tensor, Device, Kind};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenMoveDirection, Square};

lazy_static! {
    static ref DEVICE: Device = Device::cuda_if_available();
    static ref VS: nn::VarStore = nn::VarStore::new(*DEVICE);
    static ref MODEL: ChessModel = ChessModel::new(&VS.root());
}

// Constants for the input tensor
const NUM_PIECE_TYPE_BITS: u8 = 6; // 6 piece types
const NUM_COLOR_BITS: u8 = 2; // 2 colors
const NUM_BITS_PER_BOARD: u8 = NUM_PIECE_TYPE_BITS * NUM_COLOR_BITS;

const NUM_STATES_LOOKBACK: u8 = 0; // no lookback
const NUM_STATES_TO_CONSIDER: u8 = NUM_STATES_LOOKBACK + 1;

const NUM_BOARD_BITS: u8 = NUM_BITS_PER_BOARD * NUM_STATES_TO_CONSIDER; // 12 bits for board(s)

const NUM_CASTLING_BITS: u8 = 4; // 4 castling rights
const NUM_SIDE_TO_MOVE_BITS: u8 = 1; // 1 bit for side to move
const NUM_METADATA_BITS: u8 = NUM_CASTLING_BITS + NUM_SIDE_TO_MOVE_BITS; // 5 bits for metadata

const NUM_POSITION_BITS: u8 = NUM_BOARD_BITS + NUM_METADATA_BITS; // 17 8x8 planes in the input tensor

const NUM_RAY_DIRECTIONS: u8 = 8; // 8 directions for queen-like moves
const MAX_RAY_LENGTH: u8 = 7; // Maximum length of a queen-like move
const NUM_QUEEN_LIKE_MOVES: u8 = NUM_RAY_DIRECTIONS * MAX_RAY_LENGTH; // 56 possible queen-like moves

const MAX_NUM_KNIGHT_MOVES: u8 = 8; // Maximum number of knight moves

const NUM_PAWN_MOVE_DIRECTIONS: u8 = 3; // 3 possible pawn moves
const NUM_UNDERPROMOTIONS: u8 = 3; // 3 underpromotions (knight, bishop, rook)
const NUM_WAYS_OF_UNDERPROMOTION: u8 = NUM_PAWN_MOVE_DIRECTIONS * NUM_UNDERPROMOTIONS; // 9 ways of underpromotion

const NUM_TARGET_SQUARE_POSSIBILITIES: u8 = NUM_QUEEN_LIKE_MOVES + MAX_NUM_KNIGHT_MOVES + NUM_WAYS_OF_UNDERPROMOTION; // 73 of possible target squares for a move
const NUM_OUTPUT_POLICY_MOVES: usize = 64 * NUM_TARGET_SQUARE_POSSIBILITIES as usize; // 4672 possible moves for policy head
const NUM_INITIAL_CONV_OUTPUT_CHANNELS: usize = 32; // Output channels for initial conv layer

/// Checks if a move is a knight move based on its source and destination squares.
fn is_knight_jump(src_square: Square, dst_square: Square) -> bool {
    // Calculate the difference in rank and file between the source and destination
    let rank_diff = (dst_square.get_rank() as i8 - src_square.get_rank() as i8).abs();
    let file_diff = (dst_square.get_file() as i8 - src_square.get_file() as i8).abs();

    // A knight move is either (±2, ±1) or (±1, ±2)
    (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
}

/// Maps a queen-like move to an index in the policy tensor's 73 possible moves per square.
/// Index is between 0 and 64 for queen-like moves (56 different target squares, 9 possible underpromotions).
fn get_policy_index_for_queen_like_move(direction: QueenMoveDirection, distance: u8, promotion: Option<PieceType>) -> u8 {
    // Calculate the index based on the direction and distance
    let direction_index = direction as u8;
    let distance_index = distance - 1; // Distance is 1-indexed

    let promotion_index = match promotion {
        Some(PieceType::Knight) => 0,
        Some(PieceType::Bishop) => 1,
        Some(PieceType::Rook) => 2,
        _ => return direction_index * MAX_RAY_LENGTH + distance_index,
    };

    let promotion_direction_index = match direction {
        QueenMoveDirection::Up => 0,
        QueenMoveDirection::UpRight => 1,
        QueenMoveDirection::UpLeft => 2,
        _ => panic!()
    };

    NUM_QUEEN_LIKE_MOVES + promotion_direction_index * NUM_UNDERPROMOTIONS + promotion_index
}

/// Maps a knight move to an index in the policy tensor's 73 possible moves per square.
/// Index is between 65 and 72 for knight moves (8 possible moves).
fn get_policy_index_for_knight_move(direction: KnightMoveDirection) -> u8 {
    direction as u8 + NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION
}

/// Maps a move to an index in the policy tensor's 73 possible moves per square.
fn get_policy_index_for_move(mv: &Move, side_to_move: Color) -> u8 {
    // Extract destination, source, promotion, and flag from the move
    let dst_square = match side_to_move {
        Color::White => mv.get_destination(),
        Color::Black => mv.get_destination().rotated_perspective()
    };
    let src_square = match side_to_move {
        Color::White => mv.get_source(),
        Color::Black => mv.get_source().rotated_perspective()
    };
    let unvetted_promotion = mv.get_promotion();
    let flag = mv.get_flag();

    if flag == MoveFlag::NormalMove && is_knight_jump(src_square, dst_square) {
        // Knight move
        get_policy_index_for_knight_move(KnightMoveDirection::calc(src_square, dst_square))
    } else {
        // Queen-like move
        let mut distance = 0;
        let direction = QueenMoveDirection::calc_and_measure_distance(src_square, dst_square, &mut distance);

        let promotion = if flag == MoveFlag::Promotion {
            Some(unvetted_promotion)
        } else {
            None
        };

        get_policy_index_for_queen_like_move(direction, distance as u8, promotion)
    }
}

/// Generates a move mask tensor, marking legal moves with 1 and others with 0.
pub fn get_move_mask(moves: &Vec<Move>, side_to_move: Color) -> Tensor {
    // Initialize a mask tensor with shape [8, 8, 73] (8x8 board, 73 possible moves)
    let mask = Tensor::zeros(&[8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));

    for mv in moves {
        // Get the source square from which the move is made
        let src_square = match side_to_move {
            Color::White => mv.get_source(),
            Color::Black => mv.get_source().rotated_perspective()
        };

        // Determine the policy index using get_policy_index_for_move
        let policy_index = get_policy_index_for_move(mv, side_to_move);

        // Set the mask at the corresponding source square and policy index to 1
        let _ = mask.get(src_square.get_rank() as i64)
            .get(src_square.get_file() as i64)
            .get(policy_index as i64)
            .fill_(1.0);
    }

    mask
}

pub fn state_to_tensor(state: &State) -> Tensor {
    // Initialize a tensor with shape [1, 17, 8, 8], where:
    // - 1 is the batch size
    // - 17 is the number of channels
    // - 8x8 is the board size
    let tensor = Tensor::zeros(&[NUM_STATES_TO_CONSIDER as i64, NUM_POSITION_BITS as i64, 8, 8], (Kind::Float, *DEVICE));

    // Determine if we need to rotate the board
    let rotate = state.side_to_move == Color::Black;

    // Channels 0-11: Piece types for both colors
    for (i, piece_type) in PieceType::iter_pieces().enumerate() {
        // Get the bitboard mask for the specific piece type and color
        let player_piece_type_mask = state.board.color_masks[state.side_to_move as usize] & state.board.piece_type_masks[piece_type as usize];
        let opponent_piece_type_mask = state.board.color_masks[state.side_to_move.flip() as usize] & state.board.piece_type_masks[piece_type as usize];

        // Channels 0-5: Player's pieces
        for square in get_squares_from_mask_iter(player_piece_type_mask) {
            let square_from_unified_perspective = if rotate {
                square.rotated_perspective()
            } else {
                square
            };
            let _ = tensor.get(0)
                .get(piece_type as i64 - PieceType::Pawn as i64)
                .get(square_from_unified_perspective.get_rank() as i64)
                .get(square_from_unified_perspective.get_file() as i64)
                .fill_(1.);
        }

        // Channels 6-11: Opponent's pieces
        for square in get_squares_from_mask_iter(opponent_piece_type_mask) {
            let square_from_unified_perspective = if rotate {
                square.rotated_perspective()
            } else {
                square
            };
            let _ = tensor.get(0)
                .get(NUM_PIECE_TYPE_BITS as i64 + piece_type as i64 - PieceType::Pawn as i64)
                .get(square_from_unified_perspective.get_rank() as i64)
                .get(square_from_unified_perspective.get_file() as i64)
                .fill_(1.);
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
        let conv1 = nn::conv2d(vs, NUM_POSITION_BITS as i64, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64, 3, nn::ConvConfig { padding: 1, ..Default::default() }); // 17 input channels, 32 output channels

        // Two residual blocks with 32 channels each
        let residual_block1 = ResidualBlock::new(&vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64);
        let residual_block2 = ResidualBlock::new(&vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64);

        // Fully connected layers for policy and value heads
        let fc_policy = nn::linear(
            vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 8 * 8, NUM_OUTPUT_POLICY_MOVES as i64, Default::default()
        ); // Map to 4672 possible moves
        let fc_value = nn::linear(
            vs, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 8 * 8, 1, Default::default()
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
        let x = x.view([-1, NUM_POSITION_BITS as i64, 8, 8]).apply(&self.conv1).relu();

        // Pass through residual blocks
        let x = self.residual_block1.forward(&x);
        let x = self.residual_block2.forward(&x);

        // Flatten for fully connected layers
        let x = x.view([-1, NUM_INITIAL_CONV_OUTPUT_CHANNELS as i64 * 8 * 8]);

        // Policy head: Softmax over 4672 possible moves
        let policy = self.fc_policy.forward(&x).view(
            [-1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]
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

        assert_eq!(tensor.size(), [1, NUM_POSITION_BITS as i64, 8, 8]);
    }

    #[test]
    fn test_chess_model() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ChessModel::new(&vs.root());

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor);

        assert_eq!(policy.size(), [1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64]);
        assert_eq!(value.size(), [1, 1]);
    }

    #[test]
    fn test_training() {
        let vs = nn::VarStore::new(*DEVICE);
        let model = ChessModel::new(&vs.root());

        let input_tensor = state_to_tensor(&State::initial());
        let (policy, value) = model.forward(&input_tensor);

        let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
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

            let target_policy = Tensor::zeros(&[1, 8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));
            let target_value = Tensor::zeros(&[1, 1], (Kind::Float, *DEVICE));

            let policy_loss = policy.kl_div(&target_policy, tch::Reduction::Mean, false);
            let value_loss = value.mse_loss(&target_value, tch::Reduction::Mean);

            let loss = policy_loss + value_loss;

            let mut optimizer = nn::Adam::default().build(&vs, 1e-3).unwrap();
            optimizer.backward_step(&loss);
        }
    }
}