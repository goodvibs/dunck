//! Move direction related enums and functions.

use crate::utils::masks::{FILE_A, FILE_H};
use crate::utils::Square;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QueenMoveDirection {
    Up=0, Down=7,
    UpRight=1, DownLeft=6,
    Right=2, Left=5,
    DownRight=3, UpLeft=4,
}

const ALL_QUEEN_MOVE_DIRECTIONS: [QueenMoveDirection; 8] = [
    QueenMoveDirection::Up, QueenMoveDirection::Down,
    QueenMoveDirection::UpRight, QueenMoveDirection::DownLeft,
    QueenMoveDirection::Right, QueenMoveDirection::Left,
    QueenMoveDirection::DownRight, QueenMoveDirection::UpLeft
];

impl QueenMoveDirection {
    pub const fn from(value: u8) -> QueenMoveDirection {
        unsafe { std::mem::transmute::<u8, QueenMoveDirection>(value) }
    }
    
    pub const fn flip(&self) -> QueenMoveDirection {
        QueenMoveDirection::from(7u8.wrapping_sub(*self as u8))
    }
    
    pub fn iter() -> impl Iterator<Item=QueenMoveDirection> {
        ALL_QUEEN_MOVE_DIRECTIONS.iter().copied()
    }

    pub const fn calc(src_square: Square, dst_square: Square) -> QueenMoveDirection {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;

        if value_change % 8 == 0 {
            positive_direction = QueenMoveDirection::Down;
        } else if value_change % 9 == 0 {
            positive_direction = QueenMoveDirection::DownRight;
        } else if src_square.get_rank() == dst_square.get_rank() {
            positive_direction = QueenMoveDirection::Right;
        } else {
            positive_direction = QueenMoveDirection::DownLeft;
        }

        if value_change < 0 {
            positive_direction.flip()
        } else {
            positive_direction
        }
    }

    pub const fn calc_and_measure_distance(src_square: Square, dst_square: Square) -> (QueenMoveDirection, usize) {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;
        let distance_temp;

        if value_change % 8 == 0 {
            positive_direction = QueenMoveDirection::Down;
            distance_temp = value_change / 8;
        } else if value_change % 9 == 0 {
            positive_direction = QueenMoveDirection::DownRight;
            distance_temp = value_change / 9;
        } else if src_square.get_rank() == dst_square.get_rank() {
            positive_direction = QueenMoveDirection::Right;
            distance_temp = value_change;
        } else {
            positive_direction = QueenMoveDirection::DownLeft;
            distance_temp = value_change / 7;
        }

        if value_change < 0 {
            (positive_direction.flip(), -distance_temp as usize)
        } else {
            (positive_direction, distance_temp as usize)
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum KnightMoveDirection {
    TwoUpOneRight=0, TwoDownOneLeft=7,
    TwoRightOneUp=1, TwoLeftOneDown=6,
    TwoRightOneDown=2, TwoLeftOneUp=5,
    TwoDownOneRight=3, TwoUpOneLeft=4,
}

const ALL_KNIGHT_MOVE_DIRECTIONS: [KnightMoveDirection; 8] = [
    KnightMoveDirection::TwoUpOneRight, KnightMoveDirection::TwoDownOneLeft,
    KnightMoveDirection::TwoRightOneUp, KnightMoveDirection::TwoLeftOneDown,
    KnightMoveDirection::TwoRightOneDown, KnightMoveDirection::TwoLeftOneUp,
    KnightMoveDirection::TwoDownOneRight, KnightMoveDirection::TwoUpOneLeft
];

impl KnightMoveDirection {
    pub const fn flip(&self) -> KnightMoveDirection {
        KnightMoveDirection::from(7u8.wrapping_sub(*self as u8))
    }

    pub const fn from(value: u8) -> KnightMoveDirection {
        unsafe { std::mem::transmute::<u8, KnightMoveDirection>(value) }
    }
    
    pub fn iter() -> impl Iterator<Item=KnightMoveDirection> {
        ALL_KNIGHT_MOVE_DIRECTIONS.iter().copied()
    }

    pub const fn calc(src_square: Square, dst_square: Square) -> KnightMoveDirection {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;

        if value_change % 15 == 0 {
            positive_direction = KnightMoveDirection::TwoDownOneLeft;
        } else if value_change % 6 == 0 {
            positive_direction = KnightMoveDirection::TwoLeftOneDown;
        } else if value_change % 17 == 0 {
            positive_direction = KnightMoveDirection::TwoDownOneRight;
        } else {
            positive_direction = KnightMoveDirection::TwoRightOneDown;
        }

        if value_change < 0 {
            positive_direction.flip()
        } else {
            positive_direction
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_queen_direction_for_square(square: Square, direction: QueenMoveDirection) {
        let mut current_square = square;
        let mut distance = 0;
        loop {
            let next_square = match direction {
                QueenMoveDirection::Up => current_square.up(),
                QueenMoveDirection::Down => current_square.down(),
                QueenMoveDirection::Right => current_square.right(),
                QueenMoveDirection::Left => current_square.left(),
                QueenMoveDirection::UpRight => current_square.up_right(),
                QueenMoveDirection::DownLeft => current_square.down_left(),
                QueenMoveDirection::DownRight => current_square.down_right(),
                QueenMoveDirection::UpLeft => current_square.up_left(),
            };

            if let Some(next_square) = next_square {
                distance += 1;
                assert_eq!(QueenMoveDirection::calc(square, next_square), direction);
                assert_eq!(QueenMoveDirection::calc_and_measure_distance(square, next_square), (direction, distance));
                current_square = next_square;
            } else {
                break;
            }
        }
    }

    fn test_all_queen_directions_for_square(square: Square) {
        for direction in QueenMoveDirection::iter() {
            test_queen_direction_for_square(square, direction);
        }
    }

    #[test]
    fn test_queen_move_direction() {
        for square in Square::iter_all() {
            test_all_queen_directions_for_square(square);
        }
    }
    
    fn test_knight_direction_for_square(square: Square, direction: KnightMoveDirection) {
        let next_square = match direction {
            KnightMoveDirection::TwoUpOneRight => square.up().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoDownOneLeft => square.down().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneUp => square.right().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoLeftOneDown => square.left().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneDown => square.right().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoLeftOneUp => square.left().and_then(|x| x.up_left()),
            KnightMoveDirection::TwoDownOneRight => square.down().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoUpOneLeft => square.up().and_then(|x| x.up_left())
        };

        if let Some(next_square) = next_square {
            assert_eq!(KnightMoveDirection::calc(square, next_square), direction);
        }
    }
    
    fn test_all_knight_directions_for_square(square: Square) {
        for direction in KnightMoveDirection::iter() {
            test_knight_direction_for_square(square, direction);
        }
    }
    
    #[test]
    fn test_knight_move_direction() {
        for square in Square::iter_all() {
            test_all_knight_directions_for_square(square);
        }
    }
}