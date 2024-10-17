use std::panic;
use crate::attacks::{multi_pawn_attacks, single_bishop_attacks, single_king_attacks, single_knight_attacks, single_rook_attacks};
use crate::utils::{Bitboard, Color, PieceType, Square};
use pyrrhic_rs::{EngineAdapter, TBError, TableBases, WdlProbeResult};
use crate::state::{State, Termination};

#[derive(Clone)]
pub struct DunckAdapter;

impl EngineAdapter for DunckAdapter {
    fn pawn_attacks(color: pyrrhic_rs::Color, square: u64) -> u64 {
        let src_square = translate_from_le_to_be_square(square);
        multi_pawn_attacks(src_square.to_mask(), translate_from_reverse_color(color))
    }

    fn knight_attacks(square: u64) -> u64 {
        let src_square = translate_from_le_to_be_square(square);
        single_knight_attacks(src_square)
    }

    fn bishop_attacks(square: u64, occupied: u64) -> u64 {
        let src_square = translate_from_le_to_be_square(square);
        let occupied_mask = translate_from_le_to_be_u64(occupied);
        single_bishop_attacks(src_square, occupied_mask)
    }

    fn rook_attacks(square: u64, occupied: u64) -> u64 {
        let src_square = translate_from_le_to_be_square(square);
        let occupied_mask = translate_from_le_to_be_u64(occupied);
        single_rook_attacks(src_square, occupied_mask)
    }

    fn queen_attacks(square: u64, occupied: u64) -> u64 {
        let src_square = translate_from_le_to_be_square(square);
        let occupied_mask = translate_from_le_to_be_u64(occupied);
        single_rook_attacks(src_square, occupied_mask) | single_bishop_attacks(src_square, occupied_mask)
    }

    fn king_attacks(square: u64) -> u64 {
        let square = translate_from_le_to_be_square(square);
        single_king_attacks(square)
    }
}

fn translate_from_le_to_be_u64(input: u64) -> Bitboard {
    input.swap_bytes()
}

fn translate_from_le_to_be_square(input: u64) -> Square {
    if input > 63 {
        println!("Invalid square: {}", input);
        return Square::H8;
    }
    let rank = input / 8;
    let file = input % 8;
    unsafe { Square::from(((7 - rank) * 8 + file) as u8) }
}

fn translate_from_reverse_color(color: pyrrhic_rs::Color) -> Color {
    match color {
        pyrrhic_rs::Color::White => Color::Black,
        pyrrhic_rs::Color::Black => Color::White,
    }
}

impl State {
    pub fn probe_tb_wdl_safe(&self, tablebase: &TableBases<DunckAdapter>) -> Result<WdlProbeResult, TBError> {
        // Extract necessary data from `self` before entering `catch_unwind`
        let white_mask = self.board.color_masks[Color::White as usize];
        let black_mask = self.board.color_masks[Color::Black as usize];
        let king_mask = self.board.piece_type_masks[PieceType::King as usize];
        let queen_mask = self.board.piece_type_masks[PieceType::Queen as usize];
        let rook_mask = self.board.piece_type_masks[PieceType::Rook as usize];
        let bishop_mask = self.board.piece_type_masks[PieceType::Bishop as usize];
        let knight_mask = self.board.piece_type_masks[PieceType::Knight as usize];
        let pawn_mask = self.board.piece_type_masks[PieceType::Pawn as usize];
        let is_black_to_move = self.side_to_move == Color::Black;

        // Now wrap only the tablebase probing code, no `self` references inside `catch_unwind`
        let result = panic::catch_unwind(|| {
            tablebase.probe_wdl(
                white_mask,
                black_mask,
                king_mask,
                queen_mask,
                rook_mask,
                bishop_mask,
                knight_mask,
                pawn_mask,
                0,
                is_black_to_move
            )
        });

        result.unwrap_or_else(|_| Err(TBError::ProbeFailed))
    }
    
    pub fn probe_tb_wdl(&self, tablebase: &TableBases<DunckAdapter>) -> Result<WdlProbeResult, TBError> {
        println!("{}", self.to_fen());
        tablebase.probe_wdl(
            self.board.color_masks[Color::White as usize],
            self.board.color_masks[Color::Black as usize],
            self.board.piece_type_masks[PieceType::King as usize],
            self.board.piece_type_masks[PieceType::Queen as usize],
            self.board.piece_type_masks[PieceType::Rook as usize],
            self.board.piece_type_masks[PieceType::Bishop as usize],
            self.board.piece_type_masks[PieceType::Knight as usize],
            self.board.piece_type_masks[PieceType::Pawn as usize],
            0,
            self.side_to_move == Color::Black
        )
    }

    pub fn is_tb_eligible(&self) -> bool {
        let context = self.context.borrow();
        context.halfmove_clock ==  0 &&
            context.castling_rights == 0 &&
            context.double_pawn_push == -1 && // todo: temporary, will fix
            self.board.piece_type_masks[PieceType::AllPieceTypes as usize].count_ones() <= 5
    }

    pub fn update_with_tb_if_eligible(&mut self, tablebase: &TableBases<DunckAdapter>) {
        if self.is_tb_eligible() {
            match self.probe_tb_wdl_safe(tablebase) {
                Ok(result) => {
                    self.termination = match result {
                        WdlProbeResult::Loss => {
                            self.side_to_move = self.side_to_move.flip();
                            Some(Termination::Checkmate)
                        },
                        WdlProbeResult::BlessedLoss => Some(Termination::FiftyMoveRule),
                        WdlProbeResult::Draw => Some(Termination::ThreefoldRepetition),
                        WdlProbeResult::CursedWin => Some(Termination::FiftyMoveRule),
                        WdlProbeResult::Win => Some(Termination::Checkmate),
                    }
                },
                Err(_) => {
                    println!("Error probing tablebase");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_from_le_to_be_u64() {
        let input = 0x0123456789ABCDEF;
        let expected = 0xEFCDAB8967452301;
        assert_eq!(translate_from_le_to_be_u64(input), expected);
    }

    #[test]
    fn test_translate_from_le_to_be_square() {
        let input = 2; // C1
        let expected = Square::C1;
        assert_eq!(translate_from_le_to_be_square(input), expected);
    }

    #[test]
    fn test_pawn_attacks() {
        for p_color in [pyrrhic_rs::Color::White, pyrrhic_rs::Color::Black].iter() {
            for p_square in 0..64 {
                let translated= DunckAdapter::pawn_attacks(*p_color, p_square);

                let color = translate_from_reverse_color(*p_color);
                let src_square = translate_from_le_to_be_square(p_square);
                let expected = multi_pawn_attacks(src_square.to_mask(), color);

                assert_eq!(expected, translated);
            }
        }
    }

    #[test]
    fn test_knight_attacks() {
        for p_square in 0..64 {
            let translated = DunckAdapter::knight_attacks(p_square);

            let src_square = translate_from_le_to_be_square(p_square);
            let expected = single_knight_attacks(src_square);

            assert_eq!(expected, translated);
        }
    }

    #[test]
    fn test_bishop_attacks() {
        let mut rng = fastrand::Rng::new();
        for p_square in 0..64 {
            for p_occupied in [rng.u64(..), rng.u64(..), rng.u64(..)].iter() {
                let translated = DunckAdapter::bishop_attacks(p_square, *p_occupied);

                let src_square = translate_from_le_to_be_square(p_square);
                let occupied = translate_from_le_to_be_u64(*p_occupied);
                let expected = single_bishop_attacks(src_square, occupied);

                assert_eq!(expected, translated);
            }
        }
    }

    #[test]
    fn test_rook_attacks() {
        let mut rng = fastrand::Rng::new();
        for p_square in 0..64 {
            for p_occupied in [rng.u64(..), rng.u64(..), rng.u64(..)].iter() {
                let translated = DunckAdapter::rook_attacks(p_square, *p_occupied);

                let src_square = translate_from_le_to_be_square(p_square);
                let occupied = translate_from_le_to_be_u64(*p_occupied);
                let expected = single_rook_attacks(src_square, occupied);

                assert_eq!(expected, translated);
            }
        }
    }

    #[test]
    fn test_queen_attacks() {
        let mut rng = fastrand::Rng::new();
        for p_square in 0..64 {
            for p_occupied in [rng.u64(..), rng.u64(..), rng.u64(..)].iter() {
                let translated = DunckAdapter::queen_attacks(p_square, *p_occupied);

                let src_square = translate_from_le_to_be_square(p_square);
                let occupied = translate_from_le_to_be_u64(*p_occupied);
                let expected = single_rook_attacks(src_square, occupied) | single_bishop_attacks(src_square, occupied);

                assert_eq!(expected, translated);
            }
        }
    }

    #[test]
    fn test_king_attacks() {
        for p_square in 0..64 {
            let translated = DunckAdapter::king_attacks(p_square);

            let src_square = translate_from_le_to_be_square(p_square);
            let expected = single_king_attacks(src_square);

            assert_eq!(expected, translated);
        }
    }

    #[test]
    fn test_tb() {
        let tb = TableBases::<DunckAdapter>::new("src/engine/syzygy/3-4-5");
        match tb {
            Ok(tb) => {
                let fen ="8/2N2K2/8/3k4/8/8/p7/8 b - - 0 13";
                let state = State::initial();
                let result = state.probe_tb_wdl(&tb);
                println!("{:?}", result);
            },
            Err(tb) => {
                panic!("{:?}", tb);
            },
        }
    }
}