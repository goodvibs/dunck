//! Magic bitboard generation and attack calculation for sliding pieces

use crate::utils::{get_bit_combinations_iter, Bitboard};
use crate::utils::masks::{ANTIDIAGONALS, DIAGONALS, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utils::{SlidingPieceType, Square};
use static_init::dynamic;
use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};

/// The size of the attack table for rooks
const ROOK_ATTACK_TABLE_SIZE: usize = 36 * 2usize.pow(10) + 28 * 2usize.pow(11) + 4 * 2usize.pow(12);
/// The size of the attack table for bishops
const BISHOP_ATTACK_TABLE_SIZE: usize = 4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9);

const RNG_SEED: u64 = 0;

/// Precomputed masks for rook relevant squares
#[dynamic]
static ROOK_RELEVANT_MASKS: [Bitboard; 64] = {
    let mut masks = [0; 64];
    for (i, square) in Square::iter_all().enumerate() {
        masks[i] = calc_rook_relevant_mask(*square);
    }
    masks
};

/// Precomputed masks for bishop relevant squares
#[dynamic]
static BISHOP_RELEVANT_MASKS: [Bitboard; 64] = {
    let mut masks = [0; 64];
    for (i, square) in Square::iter_all().enumerate() {
        masks[i] = calc_bishop_relevant_mask(*square);
    }
    masks
};

/// Magic dictionaries for rooks
#[dynamic]
static ROOK_MAGIC_DICT: MagicDict = MagicDict::new(SlidingPieceType::Rook, ROOK_ATTACK_TABLE_SIZE);

/// Magic dictionaries for bishops
#[dynamic]
static BISHOP_MAGIC_DICT: MagicDict = MagicDict::new(SlidingPieceType::Bishop, BISHOP_ATTACK_TABLE_SIZE);

/// Calculate the relevant mask for a rook on a given square
fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.get_file_mask();
    let rank_mask = square.get_rank_mask();
    let mut res = (file_mask | rank_mask) & !square.get_mask();
    let edge_masks = [FILE_A, FILE_H, RANK_1, RANK_8];
    for edge_mask in edge_masks {
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
    }
    res
}

/// Get the precomputed relevant mask for a rook on a given square
pub fn get_rook_relevant_mask(square: Square) -> Bitboard {
    ROOK_RELEVANT_MASKS[square as usize]
}

/// Calculate the relevant mask for a bishop on a given square
fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.get_mask();
    let mut res = 0 as Bitboard;
    for &diagonal in DIAGONALS.iter() {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    for &antidiagonal in ANTIDIAGONALS.iter() {
        if antidiagonal & square_mask != 0 {
            res |= antidiagonal;
        }
    }
    res & !square_mask & !(FILE_A | FILE_H | RANK_1 | RANK_8)
}

/// Get the precomputed relevant mask for a bishop on a given square
pub fn get_bishop_relevant_mask(square: Square) -> Bitboard {
    BISHOP_RELEVANT_MASKS[square as usize]
}

/// A magic dictionary for a sliding piece
pub struct MagicDict {
    attacks: Box<[Bitboard]>,
    magic_info_for_squares: [MagicInfo; 64],
}

impl MagicDict {
    /// Initialize an empty magic dictionary
    fn init_empty(size: usize) -> Self {
        MagicDict {
            attacks: vec![0; size].into_boxed_slice(),
            magic_info_for_squares: [MagicInfo {
                relevant_mask: 0,
                magic_number: 0,
                right_shift_amount: 0,
                offset: 0
            }; 64]
        }
    }

    /// Create a new magic dictionary for a sliding piece
    pub fn new(sliding_piece: SlidingPieceType, size: usize) -> Self {
        let mut res = Self::init_empty(size);
        res.fill_magic_numbers_and_attacks(sliding_piece);
        res
    }

    /// Get the magic info for a square
    pub fn get_magic_info_for_square(&self, square: Square) -> MagicInfo {
        self.magic_info_for_squares[square as usize]
    }

    /// Calculate the attack mask for a square with a given occupied mask
    pub fn calc_attack_mask(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.get_magic_info_for_square(square);
        let magic_index = calc_magic_index(&magic_info, occupied_mask);
        self.attacks[magic_index]
    }

    /// Fill the magic numbers and attack tables for all squares
    pub fn fill_magic_numbers_and_attacks(&mut self, sliding_piece: SlidingPieceType) {
        let mut current_offset = 0;
        for square in Square::iter_all() {
            unsafe { self.fill_magic_numbers_and_attacks_for_square(*square, sliding_piece, &mut current_offset) };
        }
    }

    /// Fill the magic numbers and attack tables for a single square
    unsafe fn fill_magic_numbers_and_attacks_for_square(&mut self, square: Square, sliding_piece: SlidingPieceType, current_offset: &mut u32) -> Bitboard {
        let mut rng = fastrand::Rng::with_seed(RNG_SEED);

        let relevant_mask = match sliding_piece {
            SlidingPieceType::Rook => get_rook_relevant_mask(square),
            SlidingPieceType::Bishop => get_bishop_relevant_mask(square),
        };

        let mut magic_number: Bitboard;

        loop {
            magic_number = gen_random_magic_number(&mut rng);

            // Test if the magic number is suitable based on a quick bit-count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones() < 6 {
                continue;
            }

            let num_relevant_bits = relevant_mask.count_ones() as usize;
            let right_shift_amount = 64 - num_relevant_bits as u8;
            let mut used = vec![0 as Bitboard; 1 << num_relevant_bits];

            let magic_info = MagicInfo { relevant_mask, magic_number, right_shift_amount, offset: *current_offset };

            let mut failed = false;

            for (_i, occupied_mask) in get_bit_combinations_iter(relevant_mask).enumerate() {
                let attack_mask = match sliding_piece {
                    SlidingPieceType::Rook => manual_single_rook_attacks(square, occupied_mask),
                    SlidingPieceType::Bishop => manual_single_bishop_attacks(square, occupied_mask),
                };
                assert_ne!(attack_mask, 0);

                let used_index = calc_magic_index_without_offset(&magic_info, occupied_mask);

                // If the index in the used array is not set, store the attack mask
                if used[used_index] == 0 {
                    used[used_index] = attack_mask;
                } else if used[used_index] != attack_mask {
                    // If there's a non-constructive collision, the magic number is not suitable
                    failed = true;
                    break;
                }
            }

            if !failed {
                for (index_without_offset, attack_mask) in used.iter().enumerate() {
                    if *attack_mask == 0 {
                        continue;
                    }
                    self.attacks[index_without_offset + *current_offset as usize] = *attack_mask;
                }
                self.magic_info_for_squares[square as usize] = magic_info;
                *current_offset += used.len() as u32;
                break;
            }
        }

        magic_number
    }
}

/// Struct to store all magic-related information for a square
#[derive(Copy, Clone)]
pub struct MagicInfo {
    relevant_mask: Bitboard,
    magic_number: Bitboard,
    right_shift_amount: u8,
    offset: u32
}

/// Calculate the magic index for a square and an occupied mask
pub fn calc_magic_index_without_offset(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    let blockers = occupied_mask & magic_info.relevant_mask;
    let mut hash = blockers.wrapping_mul(magic_info.magic_number);
    hash >>= magic_info.right_shift_amount;
    hash as usize
}

/// Calculate the magic index for a square and an occupied mask
pub fn calc_magic_index(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    calc_magic_index_without_offset(magic_info, occupied_mask) + magic_info.offset as usize
}

/// Calculate the attack mask for a rook on a given square with a given occupied mask
pub fn magic_single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    ROOK_MAGIC_DICT.calc_attack_mask(src_square, occupied_mask)
}

/// Calculate the attack mask for a bishop on a given square with a given occupied mask
pub fn magic_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    BISHOP_MAGIC_DICT.calc_attack_mask(src_square, occupied_mask)
}

/// Generate a 64-bit random number with all zeros in the upper 60 bits
fn gen_lower_bits_random(rng: &mut fastrand::Rng) -> Bitboard {
    rng.u64(..) & 0xFFFF
}

/// Generate a 64-bit random number with a generally uniform distribution of set bits
fn gen_uniform_random(rng: &mut fastrand::Rng) -> Bitboard {
    gen_lower_bits_random(rng) | (gen_lower_bits_random(rng) << 16) | (gen_lower_bits_random(rng) << 32) | (gen_lower_bits_random(rng) << 48)
}

/// Generate a 64-bit random number likely to be suitable as a magic number
fn gen_random_magic_number(rng: &mut fastrand::Rng) -> Bitboard {
    gen_uniform_random(rng) & gen_uniform_random(rng) & gen_uniform_random(rng)
}

mod tests {
    use crate::attacks::{magic, manual};
    use crate::attacks::magic::{get_bishop_relevant_mask, get_rook_relevant_mask, BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
    use crate::utils::get_bit_combinations_iter;
    use crate::utils::charboard::print_bb_pretty;
    use crate::utils::{SlidingPieceType, Square};

    #[test]
    fn test_calc_rook_relevant_mask() {
        for mask in ROOK_RELEVANT_MASKS.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }

    #[test]
    fn test_calc_bishop_relevant_mask() {
        for mask in BISHOP_RELEVANT_MASKS.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }

    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        for sliding_piece in [SlidingPieceType::Rook, SlidingPieceType::Bishop] {
            for src_square in Square::iter_all() {
                let relevant_mask = match sliding_piece {
                    SlidingPieceType::Rook => get_rook_relevant_mask(*src_square),
                    SlidingPieceType::Bishop => get_bishop_relevant_mask(*src_square),
                };
                let occupied_masks_iter = get_bit_combinations_iter(relevant_mask);
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        SlidingPieceType::Rook => magic::magic_single_rook_attacks(*src_square, occupied_mask),
                        SlidingPieceType::Bishop => magic::magic_single_bishop_attacks(*src_square, occupied_mask),
                    };
                    let manual_attacks = match sliding_piece {
                        SlidingPieceType::Rook => manual::manual_single_rook_attacks(*src_square, occupied_mask),
                        SlidingPieceType::Bishop => manual::manual_single_bishop_attacks(*src_square, occupied_mask),
                    };
                    if magic_attacks != manual_attacks {
                        println!("Square mask:");
                        print_bb_pretty(src_square.get_mask());
                        println!("\nOccupied mask:");
                        print_bb_pretty(occupied_mask);
                        println!("\nMagic attacks:");
                        print_bb_pretty(magic_attacks);
                        println!("\nManual attacks:");
                        print_bb_pretty(manual_attacks);
                    }
                    assert_eq!(magic_attacks, manual_attacks);
                }
            }
        }
    }
}