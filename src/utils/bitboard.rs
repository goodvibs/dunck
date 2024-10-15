use crate::utils::Square;

pub type Bitboard = u64;

pub fn unpack_mask(mut mask: Bitboard) -> Vec<Bitboard> {
    let num_set_bits = mask.count_ones(); // Count the number of set bits
    let mut res = Vec::with_capacity(num_set_bits as usize); // Allocate vector with exact capacity needed

    while mask != 0 {
        let ls1b = mask & mask.wrapping_neg();  // Isolate the least significant set bit
        res.push(ls1b);
        mask &= !ls1b;  // Clear the least significant set bit
    }

    res
}

pub fn get_squares_from_mask(mut mask: Bitboard) -> Vec<Square> {
    let num_set_bits = mask.count_ones(); // Count the number of set bits
    let mut res = Vec::with_capacity(num_set_bits as usize); // Allocate vector with exact capacity needed

    for _ in 0..num_set_bits {
        let ls1b = mask & mask.wrapping_neg();  // Isolate the least significant set bit
        let square_index = ls1b.leading_zeros();  // Index of the set bit
        unsafe {
            res.push(Square::from(square_index as u8));
        }
        mask &= !ls1b;  // Clear the least significant set bit
    }

    res
}

#[derive(Debug, Clone)]
pub struct BitCombinationsIterator {
    set: u64,
    subset: u64,
    finished: bool,
}

impl BitCombinationsIterator {
    pub fn new(set: u64) -> Self {
        BitCombinationsIterator {
            set,
            subset: 0,
            finished: set == 0,
        }
    }
}

impl Iterator for BitCombinationsIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let current = self.subset;
        self.subset = self.subset.wrapping_sub(self.set) & self.set;

        // Once we generate the 0 subset again, we're done
        if self.subset == 0 && current != 0 {
            self.finished = true;
        }

        Some(current)
    }
}

pub fn generate_bit_combinations(mask: Bitboard) -> BitCombinationsIterator {
    BitCombinationsIterator::new(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_bb() {
        let bb: Bitboard = 0b10010100_10111100_00111011_11001101_01010101_01010000_01010000_01000001;
        let res = unpack_mask(bb);
        assert_eq!(res.len(), bb.count_ones() as usize);
        let mut bb_builder: Bitboard = 0;
        for mask in res.iter() {
            bb_builder |= *mask;
        }
        assert_eq!(bb, bb_builder);
    }
    
    #[test]
    fn test_get_squares_from_bb() {
        let bb: Bitboard = 0b10010100_10111100_00111011_11001101_01010101_01010000_01010000_01000001;
        let res = get_squares_from_mask(bb);
        assert_eq!(res.len(), bb.count_ones() as usize);
        assert_eq!(res[0], Square::H1);
        assert_eq!(res[1], Square::B1);
        assert_eq!(res[2], Square::D2);
        assert_eq!(res.last(), Some(&Square::A8));
        let mut bb_builder: Bitboard = 0;
        for square in res.iter() {
            bb_builder |= 1 << 63 - *square as u8;
        }
        assert_eq!(bb, bb_builder);
    }

    #[test]
    fn test_generate_bit_combinations() {
        // Test with an empty bitmask
        let mask = 0;
        let expected: Vec<u64> = vec![];
        let result: Vec<u64> = generate_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has one bit set
        let mask = 0b0001;
        let expected: Vec<u64> = vec![0b0000, 0b0001];
        let result: Vec<u64> = generate_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has multiple bits set
        let mask = 0b1010;
        let expected: Vec<u64> = vec![0b0000, 0b0010, 0b1000, 0b1010];
        let result: Vec<u64> = generate_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a full bitmask (all bits set for a small size)
        let mask = 0b1111;
        let expected: Vec<u64> = vec![
            0b0000, 0b0001, 0b0010, 0b0011,
            0b0100, 0b0101, 0b0110, 0b0111,
            0b1000, 0b1001, 0b1010, 0b1011,
            0b1100, 0b1101, 0b1110, 0b1111,
        ];
        let result: Vec<u64> = generate_bit_combinations(mask).collect();
        assert_eq!(result, expected);
    }
}