//! Move direction related enums and functions.

use crate::utils::Square;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QueenLikeMoveDirection {
    Up=0, Down=7,
    UpRight=1, DownLeft=6,
    Right=2, Left=5,
    DownRight=3, UpLeft=4,
}

const ALL_QUEEN_MOVE_DIRECTIONS: [QueenLikeMoveDirection; 8] = [
    QueenLikeMoveDirection::Up, QueenLikeMoveDirection::Down,
    QueenLikeMoveDirection::UpRight, QueenLikeMoveDirection::DownLeft,
    QueenLikeMoveDirection::Right, QueenLikeMoveDirection::Left,
    QueenLikeMoveDirection::DownRight, QueenLikeMoveDirection::UpLeft
];

impl QueenLikeMoveDirection {
    pub const fn from(value: u8) -> QueenLikeMoveDirection {
        unsafe { std::mem::transmute::<u8, QueenLikeMoveDirection>(value) }
    }
    
    pub const fn flip(&self) -> QueenLikeMoveDirection {
        QueenLikeMoveDirection::from(7u8.wrapping_sub(*self as u8))
    }
    
    pub fn iter() -> impl Iterator<Item=QueenLikeMoveDirection> {
        ALL_QUEEN_MOVE_DIRECTIONS.iter().copied()
    }

    pub const fn calc(src_square: Square, dst_square: Square) -> QueenLikeMoveDirection {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;

        if value_change % 8 == 0 {
            positive_direction = QueenLikeMoveDirection::Down;
        } else if value_change % 9 == 0 {
            positive_direction = QueenLikeMoveDirection::DownRight;
        } else if src_square.get_rank() == dst_square.get_rank() {
            positive_direction = QueenLikeMoveDirection::Right;
        } else {
            positive_direction = QueenLikeMoveDirection::DownLeft;
        }

        if value_change < 0 {
            positive_direction.flip()
        } else {
            positive_direction
        }
    }

    pub const fn calc_and_measure_distance(src_square: Square, dst_square: Square) -> (QueenLikeMoveDirection, u8) {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;
        let distance_temp;

        if value_change % 8 == 0 {
            positive_direction = QueenLikeMoveDirection::Down;
            distance_temp = value_change / 8;
        } else if value_change % 9 == 0 {
            positive_direction = QueenLikeMoveDirection::DownRight;
            distance_temp = value_change / 9;
        } else if src_square.get_rank() == dst_square.get_rank() {
            positive_direction = QueenLikeMoveDirection::Right;
            distance_temp = value_change;
        } else {
            positive_direction = QueenLikeMoveDirection::DownLeft;
            distance_temp = value_change / 7;
        }

        if value_change < 0 {
            (positive_direction.flip(), -distance_temp as u8)
        } else {
            (positive_direction, distance_temp as u8)
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
    
    fn test_queen_direction_for_square(square: Square, direction: QueenLikeMoveDirection) {
        let mut current_square = square;
        let mut distance = 0;
        loop {
            let next_square = match direction {
                QueenLikeMoveDirection::Up => current_square.up(),
                QueenLikeMoveDirection::Down => current_square.down(),
                QueenLikeMoveDirection::Right => current_square.right(),
                QueenLikeMoveDirection::Left => current_square.left(),
                QueenLikeMoveDirection::UpRight => current_square.up_right(),
                QueenLikeMoveDirection::DownLeft => current_square.down_left(),
                QueenLikeMoveDirection::DownRight => current_square.down_right(),
                QueenLikeMoveDirection::UpLeft => current_square.up_left(),
            };

            if let Some(next_square) = next_square {
                distance += 1;
                assert_eq!(QueenLikeMoveDirection::calc(square, next_square), direction);
                assert_eq!(QueenLikeMoveDirection::calc_and_measure_distance(square, next_square), (direction, distance));
                current_square = next_square;
            } else {
                break;
            }
        }
    }

    fn test_all_queen_directions_for_square(square: Square) {
        for direction in QueenLikeMoveDirection::iter() {
            test_queen_direction_for_square(square, direction);
        }
    }

    #[test]
    fn test_queen_move_direction() {
        for square in Square::iter_all() {
            test_all_queen_directions_for_square(*square);
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
            test_all_knight_directions_for_square(*square);
        }
    }
}