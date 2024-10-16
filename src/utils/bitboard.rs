use crate::utils::Square;

pub type Bitboard = u64;

#[derive(Debug, Clone)]
pub struct SetBitMaskIterator {
    mask: Bitboard,
}

impl From<Bitboard> for SetBitMaskIterator {
    fn from(mask: Bitboard) -> Self {
        SetBitMaskIterator {
            mask,
        }
    }
}

impl Iterator for SetBitMaskIterator {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            return None;
        }

        let ls1b = self.mask & self.mask.wrapping_neg();  // Isolate the least significant set bit
        self.mask &= !ls1b;  // Clear the least significant set bit

        Some(ls1b)
    }
}

pub fn get_set_bit_mask_iter(mask: Bitboard) -> SetBitMaskIterator {
    mask.into()
}

#[derive(Debug, Clone)]
pub struct SquaresFromMaskIterator {
    mask: Bitboard,
}

impl From<Bitboard> for SquaresFromMaskIterator {
    fn from(mask: Bitboard) -> Self {
        SquaresFromMaskIterator {
            mask,
        }
    }
}

impl Iterator for SquaresFromMaskIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            return None;
        }

        let ls1b = self.mask & self.mask.wrapping_neg();  // Isolate the least significant set bit
        self.mask &= !ls1b;  // Clear the least significant set bit
        let square_index = ls1b.leading_zeros();  // Index of the set bit

        unsafe {
            Some(Square::from(square_index as u8))
        }
    }
}

pub fn get_squares_from_mask_iter(mask: Bitboard) -> SquaresFromMaskIterator {
    mask.into()
}

#[derive(Debug, Clone)]
pub struct BitCombinationsIterator {
    set: Bitboard,
    subset: Bitboard,
    finished: bool,
}

impl From<Bitboard> for BitCombinationsIterator {
    fn from(set: Bitboard) -> Self {
        BitCombinationsIterator {
            set,
            subset: 0,
            finished: set == 0,
        }
    }
}

impl Iterator for BitCombinationsIterator {
    type Item = Bitboard;

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

pub fn get_bit_combinations_iter(mask: Bitboard) -> BitCombinationsIterator {
    mask.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bit_combinations() {
        // Test with an empty bitmask
        let mask = 0;
        let expected: Vec<Bitboard> = vec![];
        let result: Vec<Bitboard> = get_bit_combinations_iter(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has one bit set
        let mask = 0b0001;
        let expected: Vec<Bitboard> = vec![0b0000, 0b0001];
        let result: Vec<Bitboard> = get_bit_combinations_iter(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has multiple bits set
        let mask = 0b1010;
        let expected: Vec<Bitboard> = vec![0b0000, 0b0010, 0b1000, 0b1010];
        let result: Vec<Bitboard> = get_bit_combinations_iter(mask).collect();
        assert_eq!(result, expected);

        // Test with a full bitmask (all bits set for a small size)
        let mask = 0b1111;
        let expected: Vec<Bitboard> = vec![
            0b0000, 0b0001, 0b0010, 0b0011,
            0b0100, 0b0101, 0b0110, 0b0111,
            0b1000, 0b1001, 0b1010, 0b1011,
            0b1100, 0b1101, 0b1110, 0b1111,
        ];
        let result: Vec<Bitboard> = get_bit_combinations_iter(mask).collect();
        assert_eq!(result, expected);
    }
}