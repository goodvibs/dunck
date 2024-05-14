use crate::miscellaneous::Square;

pub type Bitboard = u64;

pub fn unpack_bb(mut bb: Bitboard) -> Vec<Bitboard> {
    let num_set_bits = bb.count_ones(); // Count the number of set bits
    let mut res = Vec::with_capacity(num_set_bits as usize); // Allocate vector with exact capacity needed

    while bb != 0 {
        let ls1b = bb & bb.wrapping_neg();  // Isolate the least significant set bit
        res.push(ls1b);
        bb &= !ls1b;  // Clear the least significant set bit
    }

    res
}

pub fn get_squares_from_bb(mut bb: Bitboard) -> Vec<Square> {
    let num_set_bits = bb.count_ones(); // Count the number of set bits
    let mut res = Vec::with_capacity(num_set_bits as usize); // Allocate vector with exact capacity needed

    while bb != 0 {
        let ls1b = bb & bb.wrapping_neg();  // Isolate the least significant set bit
        let square_index = ls1b.leading_zeros();  // Index of the set bit
        unsafe {
            res.push(Square::from(square_index as u8));
        }
        bb &= !ls1b;  // Clear the least significant set bit
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpack_bb() {
        let bb: Bitboard = 0b10010100_10111100_00111011_11001101_01010101_01010000_01010000_01000001;
        let res = unpack_bb(bb);
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
        let res = get_squares_from_bb(bb);
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
}