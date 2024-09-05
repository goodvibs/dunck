use std::collections::HashSet;
use std::fs::{metadata, File};
use std::io::{BufReader, BufWriter};
use std::iter::zip;
use lazy_static::lazy_static;
use crate::bitboard::{generate_bit_combinations, BitCombinationsIterator, Bitboard};
use crate::charboard::print_bb;
use crate::manual_attacks;
use crate::masks::{ANTIDIAGONALS, DIAGONALS, FILE_A, FILE_B, FILE_H, RANK_1, RANK_8};
use crate::miscellaneous::{PieceType, Square};

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

    static ref rook_magic_dict: MagicDict = MagicDict::new(PieceType::Rook);
    
    static ref bishop_magic_dict: MagicDict = MagicDict::new(PieceType::Bishop);
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

const SIZE_PER_SQUARE: usize = 4096;
const TOTAL_ARRAY_SIZE: usize = SIZE_PER_SQUARE * 64;

pub struct MagicDict {
    attacks: Vec<Bitboard>,
    magic_info_for_squares: [MagicInfo; 64],
}

impl MagicDict {
    pub fn new_empty() -> Self {
        MagicDict {
            attacks: vec![0; TOTAL_ARRAY_SIZE],
            magic_info_for_squares: [MagicInfo { relevant_mask: 0, magic_number: 0, right_shift_amount: 0}; 64]
        }
    }

    pub fn new(sliding_piece: PieceType) -> Self {
        let mut res = MagicDict::new_empty();
        res.fill_magic_numbers_and_attacks(sliding_piece);
        res
    }

    pub fn get_magic_info_for_square(&self, square: Square) -> MagicInfo {
        self.magic_info_for_squares[square as usize]
    }

    pub fn calc_attack_mask(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.get_magic_info_for_square(square);
        let magic_index = calc_magic_index(&magic_info, occupied_mask);
        assert!(magic_index < SIZE_PER_SQUARE);
        self.attacks[square as usize * SIZE_PER_SQUARE + magic_index]
    }

    pub fn fill_magic_numbers_and_attacks(&mut self, sliding_piece: PieceType) {
        for square in Square::iter_all() {
            unsafe { self.fill_magic_numbers_and_attacks_for_square(square, sliding_piece) };
        }
    }

    unsafe fn fill_magic_numbers_and_attacks_for_square(&mut self, square: Square, sliding_piece: PieceType) -> Bitboard {
        let relevant_mask = match sliding_piece {
            PieceType::Rook => get_rook_relevant_mask(square),
            PieceType::Bishop => get_bishop_relevant_mask(square),
            _ => panic!("Invalid sliding piece type")
        };

        let mut magic_number: Bitboard;

        loop {
            magic_number = gen_random_magic_number();

            // Test if the magic number is suitable based on a quick bit-count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones() < 6 {
                continue;
            }

            let magic_info = MagicInfo { relevant_mask, magic_number, right_shift_amount: 64 - relevant_mask.count_ones() as u8 };

            let mut failed = false;

            // Clear the used array for the current iteration
            let mut used = [0 as Bitboard; SIZE_PER_SQUARE];

            for (i, occupied_mask) in generate_bit_combinations(relevant_mask).enumerate() {
                let attack_mask = match sliding_piece {
                    PieceType::Rook => manual_attacks::single_rook_attacks(square.to_mask(), occupied_mask),
                    PieceType::Bishop => manual_attacks::single_bishop_attacks(square.to_mask(), occupied_mask),
                    _ => panic!("Invalid sliding piece type")
                };
                assert_ne!(attack_mask, 0);

                let index = calc_magic_index(&magic_info, occupied_mask);

                // If the index in the used array is not set, store the attack mask
                if used[index] == 0 {
                    used[index] = attack_mask;
                } else if used[index] != attack_mask {
                    // If there's a non-constructive collision, the magic number is not suitable
                    failed = true;
                    break;
                }
            }

            if !failed {
                for (index, attack_mask) in used.iter().enumerate() {
                    if *attack_mask == 0 {
                        continue;
                    }
                    self.attacks[square as usize * SIZE_PER_SQUARE + index] = *attack_mask;
                }
                self.magic_info_for_squares[square as usize] = magic_info;
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
    right_shift_amount: u8
}

fn manual_sliding_piece_attacks(src_mask: Bitboard, occupied_mask: Bitboard, sliding_piece: PieceType) -> Bitboard {
    match sliding_piece {
        PieceType::Rook => manual_attacks::single_rook_attacks(src_mask, occupied_mask),
        PieceType::Bishop => manual_attacks::single_bishop_attacks(src_mask, occupied_mask),
        _ => panic!("Invalid sliding piece type")
    }
}

pub fn calc_magic_index(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    let blockers = occupied_mask & magic_info.relevant_mask;
    let hash = blockers.wrapping_mul(magic_info.magic_number);
    (hash >> magic_info.right_shift_amount) as usize
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
    use std::thread;
    use crate::charboard::{print_bb, print_bb_pretty};
    use super::*;

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
        for sliding_piece in [PieceType::Rook, PieceType::Bishop] {
            for square in Square::iter_all() {
                let src_mask = square.to_mask();
                let relevant_mask = match sliding_piece {
                    PieceType::Rook => get_rook_relevant_mask(square),
                    PieceType::Bishop => get_bishop_relevant_mask(square),
                    _ => panic!("Invalid sliding piece type")
                };
                let occupied_masks_iter = generate_bit_combinations(relevant_mask);
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        PieceType::Rook => unsafe { single_rook_attacks(src_mask, occupied_mask) },
                        PieceType::Bishop => unsafe { single_bishop_attacks(src_mask, occupied_mask) },
                        _ => panic!("Invalid sliding piece type")
                    };
                    let manual_attacks = manual_sliding_piece_attacks(src_mask, occupied_mask, sliding_piece);
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