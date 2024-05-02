pub type Bitboard = u64;

pub fn unpack_bb(mut bb: Bitboard) -> Vec<Bitboard> {
    let mut res: Vec<Bitboard> = Vec::with_capacity(64);
    while bb != 0 {
        let lsb = 1 << bb.trailing_zeros();
        res.push(lsb);
        bb ^= lsb;
    }
    res
}

pub fn bb_to_square_indices(mut bb: Bitboard) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::with_capacity(64);
    while bb != 0 {
        let msb_index = bb.leading_zeros();
        res.push(msb_index as u8);
        bb ^= !0 >> msb_index;
    }
    res
}