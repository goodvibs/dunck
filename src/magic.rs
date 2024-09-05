use crate::bitboard::{generate_bit_combinations, Bitboard};
use crate::manual_attacks;
use crate::masks::{ANTIDIAGONALS, DIAGONALS, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::miscellaneous::{SlidingPieceType, Square};
use lazy_static::lazy_static;

lazy_static! {
    static ref ROOK_RELEVANT_MASKS: [Bitboard; 64] = {
        let mut masks = [0; 64];
        for (i, square) in Square::iter_all().enumerate() {
            masks[i] = calc_rook_relevant_mask(square);
        }
        masks
    };

    static ref BISHOP_RELEVANT_MASKS: [Bitboard; 64] = {
        let mut masks = [0; 64];
        for (i, square) in Square::iter_all().enumerate() {
            masks[i] = calc_bishop_relevant_mask(square);
        }
        masks
    };

    static ref rook_magic_dict: MagicDict = MagicDict::new(SlidingPieceType::Rook);

    static ref bishop_magic_dict: MagicDict = MagicDict::new(SlidingPieceType::Bishop);
}

fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.get_file_mask();
    let rank_mask = square.get_rank_mask();
    let mut res = (file_mask | rank_mask) & !square.to_mask();
    let edge_masks = [FILE_A, FILE_H, RANK_1, RANK_8];
    for edge_mask in edge_masks {
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
    }
    res
}

pub fn get_rook_relevant_mask(square: Square) -> Bitboard {
    ROOK_RELEVANT_MASKS[square as usize]
}

fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.to_mask();
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

pub fn get_bishop_relevant_mask(square: Square) -> Bitboard {
    BISHOP_RELEVANT_MASKS[square as usize]
}

pub struct MagicDict {
    attacks: Vec<Bitboard>,
    magic_info_for_squares: [MagicInfo; 64],
}

impl MagicDict {
    fn init_empty(attacks_array_size: usize) -> Self {
        MagicDict {
            attacks: vec![0; attacks_array_size],
            magic_info_for_squares: [MagicInfo { relevant_mask: 0, magic_number: 0, right_shift_amount: 0, offset: 0}; 64]
        }
    }

    pub fn new(sliding_piece: SlidingPieceType) -> Self {
        let mut res = match sliding_piece {
            SlidingPieceType::Rook => MagicDict::init_empty(60 * 2usize.pow(11) + 4 * 2usize.pow(12)),
            SlidingPieceType::Bishop => MagicDict::init_empty(4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9)),
        };
        res.fill_magic_numbers_and_attacks(sliding_piece);
        res
    }

    pub fn get_magic_info_for_square(&self, square: Square) -> MagicInfo {
        self.magic_info_for_squares[square as usize]
    }

    pub fn calc_attack_mask(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.get_magic_info_for_square(square);
        let magic_index = calc_magic_index(&magic_info, occupied_mask);
        self.attacks[magic_index]
    }

    pub fn fill_magic_numbers_and_attacks(&mut self, sliding_piece: SlidingPieceType) {
        let mut current_offset = 0;
        for square in Square::iter_all() {
            unsafe { self.fill_magic_numbers_and_attacks_for_square(square, sliding_piece, &mut current_offset) };
        }
    }

    unsafe fn fill_magic_numbers_and_attacks_for_square(&mut self, square: Square, sliding_piece: SlidingPieceType, current_offset: &mut u32) -> Bitboard {
        let relevant_mask = match sliding_piece {
            SlidingPieceType::Rook => get_rook_relevant_mask(square),
            SlidingPieceType::Bishop => get_bishop_relevant_mask(square),
        };

        let mut magic_number: Bitboard;

        loop {
            magic_number = gen_random_magic_number();

            // Test if the magic number is suitable based on a quick bit-count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones() < 6 {
                continue;
            }

            let num_relevant_bits = relevant_mask.count_ones() as usize;
            let right_shift_amount = 64 - num_relevant_bits as u8;
            let mut used = vec![0 as Bitboard; 1 << num_relevant_bits];

            let magic_info = MagicInfo { relevant_mask, magic_number, right_shift_amount, offset: *current_offset };

            let mut failed = false;

            for (i, occupied_mask) in generate_bit_combinations(relevant_mask).enumerate() {
                let attack_mask = match sliding_piece {
                    SlidingPieceType::Rook => manual_attacks::single_rook_attacks(square.to_mask(), occupied_mask),
                    SlidingPieceType::Bishop => manual_attacks::single_bishop_attacks(square.to_mask(), occupied_mask),
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

#[derive(Copy, Clone)]
pub struct MagicInfo {
    relevant_mask: Bitboard,
    magic_number: Bitboard,
    right_shift_amount: u8,
    offset: u32
}

pub fn calc_magic_index_without_offset(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    let blockers = occupied_mask & magic_info.relevant_mask;
    let mut hash = blockers.wrapping_mul(magic_info.magic_number);
    hash >>= magic_info.right_shift_amount;
    hash as usize
}

pub fn calc_magic_index(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    calc_magic_index_without_offset(magic_info, occupied_mask) + magic_info.offset as usize
}

pub unsafe fn single_rook_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    let src_square = Square::from(src_mask.leading_zeros() as u8);
    rook_magic_dict.calc_attack_mask(src_square, occupied_mask)
}

pub unsafe fn single_bishop_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    let src_square = Square::from(src_mask.leading_zeros() as u8);
    bishop_magic_dict.calc_attack_mask(src_square, occupied_mask)
}

fn gen_random_magic_number() -> Bitboard {
    fastrand::u64(..) & fastrand::u64(..) & fastrand::u64(..)
}

mod tests {
    use super::*;
    use crate::charboard::print_bb_pretty;

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
            for square in Square::iter_all() {
                let src_mask = square.to_mask();
                let relevant_mask = match sliding_piece {
                    SlidingPieceType::Rook => get_rook_relevant_mask(square),
                    SlidingPieceType::Bishop => get_bishop_relevant_mask(square),
                };
                let occupied_masks_iter = generate_bit_combinations(relevant_mask);
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        SlidingPieceType::Rook => unsafe { single_rook_attacks(src_mask, occupied_mask) },
                        SlidingPieceType::Bishop => unsafe { single_bishop_attacks(src_mask, occupied_mask) },
                    };
                    let manual_attacks = match sliding_piece {
                        SlidingPieceType::Rook => manual_attacks::single_rook_attacks(src_mask, occupied_mask),
                        SlidingPieceType::Bishop => manual_attacks::single_bishop_attacks(src_mask, occupied_mask),
                    };
                    if magic_attacks != manual_attacks {
                        println!("Square mask:");
                        print_bb_pretty(src_mask);
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