use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum PgnParseError {
    InvalidTag(String),
    IncorrectMoveNumber(String),
    IllegalMove(String),
    InvalidComment(String),
    InvalidVariationStart(String),
    InvalidVariationClosure(String),
    InvalidToken(String),
    InvalidResult(String),
    InvalidTagPlacement(String),
    InvalidResultPlacement(String),
}

impl Display for PgnParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnParseError::InvalidTag(tag) => write!(f, "Invalid tag: {}", tag),
            PgnParseError::IncorrectMoveNumber(mov) => write!(f, "Incorrect move number: {}", mov),
            PgnParseError::IllegalMove(mov) => write!(f, "Illegal move: {}", mov),
            PgnParseError::InvalidComment(comment) => write!(f, "Invalid comment: {}", comment),
            PgnParseError::InvalidVariationStart(variation) => write!(f, "Invalid variation start: {}", variation),
            PgnParseError::InvalidVariationClosure(variation) => write!(f, "Unfinished variation: {}", variation),
            PgnParseError::InvalidToken(token) => write!(f, "Invalid token: {}", token),
            PgnParseError::InvalidResult(result) => write!(f, "Invalid result: {}", result),
            PgnParseError::InvalidResultPlacement(result) => write!(f, "Invalid result placement: {}", result),
            PgnParseError::InvalidTagPlacement(tag) => write!(f, "Invalid tag placement: {}", tag),
        }
    }
}

impl Error for PgnParseError {}