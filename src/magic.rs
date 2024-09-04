use std::collections::HashSet;
use std::fs::{metadata, File};
use std::io::{BufReader, BufWriter};
use std::iter::zip;
use lazy_static::lazy_static;
use crate::bitboard::{generate_bit_combinations, BitCombinationsIterator, Bitboard};
use crate::charboard::print_bb;
use crate::manual_attacks;
use crate::masks::{FILE_A, FILE_B, FILE_H, RANK_1, RANK_8};
use crate::miscellaneous::Square;

static mut magic_rook_attacks: [[Bitboard; 4096]; 64] = [[0; 4096]; 64];
static mut magic_rook_table: [MagicInfo; 64] = [MagicInfo { relevant_mask: 0, magic_number: 0, right_shift_amount: 0, offset_amount: 0 }; 64];

#[derive(Copy, Clone)]
pub struct MagicInfo {
    relevant_mask: Bitboard,
    magic_number: Bitboard,
    right_shift_amount: u8,
    offset_amount: u8,
}

lazy_static! {
    static ref ROOK_RELEVANT_MASKS: [Bitboard; 64] = {
        let mut masks = [0; 64];
        for (i, square) in Square::iter_all().enumerate() {
            masks[i] = calc_rook_relevant_mask(square);
        }
        masks
    };
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

pub fn calc_magic_index(magic_info: &MagicInfo, occupied_mask: Bitboard) -> usize {
    let blockers = occupied_mask & magic_info.relevant_mask;
    let hash = blockers.wrapping_mul(magic_info.magic_number);
    let index = (hash >> magic_info.right_shift_amount) as usize;
    (magic_info.offset_amount as usize + index) % 4096
}

pub unsafe fn single_rook_attacks(src_mask: Bitboard, occupied_mask: Bitboard) -> Bitboard {
    let src_square = Square::from(src_mask.leading_zeros() as u8);
    let magic_info = &magic_rook_table[src_square as usize];
    let magic_index = calc_magic_index(magic_info, occupied_mask);
    magic_rook_attacks[src_square as usize][magic_index]
}

fn gen_random_magic_number() -> Bitboard {
    fastrand::u64(..) & fastrand::u64(..) & fastrand::u64(..)
}

unsafe fn fill_magic_numbers_and_attacks() {
    for square in Square::iter_all() {
        fill_magic_number_and_attacks_for_square(square);
    }
    println!("Magic numbers and attacks filled");
}

unsafe fn fill_magic_number_and_attacks_for_square(square: Square) -> Bitboard {
    let relevant_mask = get_rook_relevant_mask(square);
    let occupied_masks_iter = generate_bit_combinations(relevant_mask);
    let attacks_for_occupied_masks = occupied_masks_iter.clone()
        .map(|occupied_mask| {
            let src_mask = square.to_mask();
            manual_attacks::single_rook_attacks(src_mask, occupied_mask)
        })
        .collect::<Vec<Bitboard>>();

    let mut magic_number: Bitboard;
    let offset_amount = square as u8;

    loop {
        magic_number = gen_random_magic_number();

        // Test if the magic number is suitable based on a quick bit-count heuristic
        if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones() < 6 {
            continue;
        }

        let relevant_mask_set_bits = relevant_mask.count_ones() as u8;
        let magic_info = MagicInfo { relevant_mask, magic_number, right_shift_amount: 64 - relevant_mask_set_bits, offset_amount };

        let mut failed = false;

        // Clear the used array for the current iteration
        let mut used = [0 as Bitboard; 4096];

        for (i, (occupied_mask, attack_mask)) in zip(occupied_masks_iter.clone(), &attacks_for_occupied_masks).enumerate() {
            assert_ne!(*attack_mask, 0);
            
            let index = calc_magic_index(&magic_info, occupied_mask);

            // If the index in the used array is not set, store the attack mask
            if used[index] == 0 {
                used[index] = *attack_mask;
            } else if used[index] != *attack_mask {
                // If there's a non-constructive collision, the magic number is not suitable
                failed = true;
                break;
            }
        }

        if !failed {
            for (i, attack_mask) in used.iter().enumerate() {
                if *attack_mask == 0 {
                    continue;
                }
                magic_rook_attacks[square as usize][i] = *attack_mask;
            }
            magic_rook_table[square as usize] = magic_info;
            println!("Magic number found for square {:?}: {:064b}", square, magic_number);
            break;
        }
    }

    magic_number
}

mod tests {
    use crate::charboard::print_bb;
    use super::*;

    #[test]
    fn test_calc_rook_relevant_mask() {
        for mask in ROOK_RELEVANT_MASKS.iter() {
            print_bb(*mask);
            println!();
        }
    }

    #[test]
    fn test_find_magic_number_for_square() {
        let square = Square::B6;
        let magic = unsafe { fill_magic_number_and_attacks_for_square(square) };
        println!("{:064b}", magic);
    }
    
    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        unsafe { fill_magic_numbers_and_attacks() };
    }
}