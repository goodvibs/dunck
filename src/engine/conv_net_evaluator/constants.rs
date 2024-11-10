// Constants for the input tensor
pub const NUM_PIECE_TYPE_BITS: u8 = 6; // 6 piece types
pub const NUM_COLOR_BITS: u8 = 2; // 2 colors
pub const NUM_BITS_PER_BOARD: u8 = NUM_PIECE_TYPE_BITS * NUM_COLOR_BITS;

pub const NUM_STATES_LOOKBACK: u8 = 0; // no lookback
pub const NUM_STATES_TO_CONSIDER: u8 = NUM_STATES_LOOKBACK + 1;

pub const NUM_BOARD_BITS: u8 = NUM_BITS_PER_BOARD * NUM_STATES_TO_CONSIDER; // 12 bits for board(s)

pub const NUM_CASTLING_BITS: u8 = 4; // 4 castling rights
pub const NUM_SIDE_TO_MOVE_BITS: u8 = 1; // 1 bit for side to move
pub const NUM_METADATA_BITS: u8 = NUM_CASTLING_BITS + NUM_SIDE_TO_MOVE_BITS; // 5 bits for metadata

pub const NUM_POSITION_BITS: u8 = NUM_BOARD_BITS + NUM_METADATA_BITS; // 17 8x8 planes in the input tensor

pub const NUM_RAY_DIRECTIONS: u8 = 8; // 8 directions for queen-like moves
pub const MAX_RAY_LENGTH: u8 = 7; // Maximum length of a queen-like move
pub const NUM_QUEEN_LIKE_MOVES: u8 = NUM_RAY_DIRECTIONS * MAX_RAY_LENGTH; // 56 possible queen-like moves

pub const MAX_NUM_KNIGHT_MOVES: u8 = 8; // Maximum number of knight moves

pub const NUM_PAWN_MOVE_DIRECTIONS: u8 = 3; // 3 possible pawn moves
pub const NUM_UNDERPROMOTIONS: u8 = 3; // 3 underpromotions (knight, bishop, rook)
pub const NUM_WAYS_OF_UNDERPROMOTION: u8 = NUM_PAWN_MOVE_DIRECTIONS * NUM_UNDERPROMOTIONS; // 9 ways of underpromotion

pub const NUM_TARGET_SQUARE_POSSIBILITIES: u8 = NUM_QUEEN_LIKE_MOVES + MAX_NUM_KNIGHT_MOVES + NUM_WAYS_OF_UNDERPROMOTION; // 73 of possible target squares for a move
pub const NUM_OUTPUT_POLICY_MOVES: usize = 64 * NUM_TARGET_SQUARE_POSSIBILITIES as usize; // 4672 possible moves for policy head
pub const NUM_INITIAL_CONV_OUTPUT_CHANNELS: usize = 32; // Output channels for initial conv layer