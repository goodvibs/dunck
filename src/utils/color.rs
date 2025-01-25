#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White=0, Black=1
}

impl Color {
    pub const fn from(is_black: bool) -> Color {
        unsafe { std::mem::transmute::<bool, Color>(is_black) }
    }

    pub const fn flip(&self) -> Color {
        unsafe { std::mem::transmute::<u8, Color>(!(*self as u8)) }
    }

    pub fn iter() -> impl Iterator<Item = Color> {
        (0..=1).map(|n| unsafe { std::mem::transmute::<u8, Color>(n) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        assert_eq!(Color::White as u8, 0);
        assert_eq!(Color::Black as u8, 1);
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
        assert_eq!(Color::from(false), Color::White);
        assert_eq!(Color::from(true), Color::Black);
    }
}