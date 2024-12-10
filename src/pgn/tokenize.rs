use std::iter::Peekable;
use std::str::Chars;
use crate::pgn::error::PgnParseError;

/// Represents a token in a PGN string
#[derive(Debug, PartialEq, Clone)]
pub enum PgnToken {
    Tag(String),                       // Represents a tag (e.g., "[Event "F/S Return Match"]")
    Move(String),                      // Represents a move (e.g., "e4", "Nf3#")
    MoveNumberAndPeriods(u16, usize),  // Represents a move number (e.g., "1", "2")
    StartVariation,                    // Represents the start of a variation ('(')
    EndVariation,                      // Represents the end of a variation (')')
    Comment(String),                   // Represents a comment (e.g., "{This is a comment}")
    Annotation(String),                // Represents an annotation (e.g., "!", "?", "!?", etc.)
    Result(String),                    // Represents a game result (e.g., "1-0", "0-1", "1/2-1/2", "*")
}

/// Tokenizes a PGN string into a list of PgnTokens
pub fn tokenize_pgn(pgn: &str) -> Result<Vec<PgnToken>, PgnParseError> {
    let mut tokens = Vec::new();

    // Create iterator over characters
    let mut chars = pgn.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            _ if ch.is_ascii_whitespace() => {
                // Skip whitespace
                chars.next();
            }
            '[' => {
                // Start of a tag
                chars.next(); // Consume '['
                let tag = collect_until(&mut chars, |c| c == ']');
                if None == chars.next() { // Consume ']'
                    return Err(PgnParseError::InvalidTag(tag));
                }
                tokens.push(PgnToken::Tag(tag));
            }
            '(' => {
                // Start of a variation
                tokens.push(PgnToken::StartVariation);
                chars.next();
            }
            ')' => {
                // End of a variation
                tokens.push(PgnToken::EndVariation);
                chars.next();
            }
            '{' => {
                // Comment starts
                chars.next(); // Consume '{'
                let comment = collect_until(&mut chars, |c| c == '}');
                if None == chars.next() { // Consume '}'
                    return Err(PgnParseError::InvalidComment(comment));
                }
                tokens.push(PgnToken::Comment(comment));
            }
            '!' | '?' | '$' => {
                // Annotation (like "!", "!?", "$19" etc.)
                let annotation = collect_until(&mut chars, |c| c.is_ascii_whitespace());
                tokens.push(PgnToken::Annotation(annotation));
            }
            '*' => {
                // Indicates an incomplete game
                tokens.push(PgnToken::Result("*".to_string()));
                chars.next();
            }
            _ if ch.is_numeric() => {
                // Could be a move number or a result
                let move_number_or_result = collect_until(&mut chars, |c| c == '.' || c.is_ascii_whitespace());
                if move_number_or_result.contains('-') {
                    tokens.push(PgnToken::Result(move_number_or_result));
                }
                else if let Ok(num) = move_number_or_result.parse::<u16>() {
                    let periods = collect_until(&mut chars, |c| c != '.');
                    tokens.push(PgnToken::MoveNumberAndPeriods(num, periods.len()));
                }
                else {
                    return Err(PgnParseError::InvalidToken(move_number_or_result));
                }
            }
            _ if ch.is_alphabetic() => {
                // Assume it's a move (e.g., "e4", "Nf3", "O-O", etc.)
                let mv = collect_until(&mut chars, |c| c.is_ascii_whitespace());
                tokens.push(PgnToken::Move(mv));
            }
            _ => {
                // Invalid token
                let invalid = collect_until(&mut chars, |c| c.is_ascii_whitespace());
                return Err(PgnParseError::InvalidToken(invalid));
            }
        }
    }

    Ok(tokens)
}

/// Collects characters from the iterator until a condition is met or the iterator ends
fn collect_until(chars: &mut Peekable<Chars>, until_condition: fn(char) -> bool) -> String {
    let mut content = String::new();

    while let Some(&ch) = chars.peek() {
        if until_condition(ch) {
            break;
        }

        content.push(ch);
        chars.next();
    }

    content
}

#[cfg(test)]
mod tests {
    use crate::pgn::PgnToken::{Move, MoveNumberAndPeriods, Result, Tag};
    use super::*;

    #[test]
    fn test_tokenize_pgn() {
        let pgn = r#"
            [Event "F/S Return Match"]
            [Site "Belgrade, Serbia JUG"]
            [Date "1992.11.04"]
            [Round "29"]
            [White "Fischer, Robert J."]
            [Black "Spassky, Boris V."]
            [Result "1/2-1/2"]
            
            1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
            4. Ba4 Nf6 5. O-O Be7 6. Re1 b5
            7. Bb3 d6 8. c3 O-O 9. h3 Nb8
            10. d4 Nbd7 11. c4 c6 12. cxb5 axb5
            13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6
            16. Bh4 c5 17. dxe5 Nxe4 18. Bxe7 Qxe7
            19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4
            22. Bxc4 Nb6 23. Ne5 Rae8 24. Bxf7+ Rxf7
            25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5
            28. Qxg5 hxg5 29. b3 Ke6 30. a3 Kd6
            31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8
            34. Kf2 Bf5 35. Ra7 g6 36. Ra6+ Kc5
            37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5
            40. Rd6 Kc5 41. Ra6 Nf2 42. g4 Bd3
            43. Re6 1/2-1/2
        "#;

        let tokens = tokenize_pgn(pgn).unwrap();
        
        assert_eq!(
            tokens,
            [
                Tag("Event \"F/S Return Match\"".parse().unwrap()),
                Tag("Site \"Belgrade, Serbia JUG\"".parse().unwrap()),
                Tag("Date \"1992.11.04\"".parse().unwrap()),
                Tag("Round \"29\"".parse().unwrap()),
                Tag("White \"Fischer, Robert J.\"".parse().unwrap()),
                Tag("Black \"Spassky, Boris V.\"".parse().unwrap()),
                Tag("Result \"1/2-1/2\"".parse().unwrap()),
                MoveNumberAndPeriods(1, 1),
                Move("e4".parse().unwrap()),
                Move("e5".parse().unwrap()),
                MoveNumberAndPeriods(2, 1),
                Move("Nf3".parse().unwrap()),
                Move("Nc6".parse().unwrap()),
                MoveNumberAndPeriods(3, 1),
                Move("Bb5".parse().unwrap()),
                Move("a6".parse().unwrap()),
                MoveNumberAndPeriods(4, 1),
                Move("Ba4".parse().unwrap()),
                Move("Nf6".parse().unwrap()),
                MoveNumberAndPeriods(5, 1),
                Move("O-O".parse().unwrap()),
                Move("Be7".parse().unwrap()),
                MoveNumberAndPeriods(6, 1),
                Move("Re1".parse().unwrap()),
                Move("b5".parse().unwrap()),
                MoveNumberAndPeriods(7, 1),
                Move("Bb3".parse().unwrap()),
                Move("d6".parse().unwrap()),
                MoveNumberAndPeriods(8, 1),
                Move("c3".parse().unwrap()),
                Move("O-O".parse().unwrap()),
                MoveNumberAndPeriods(9, 1),
                Move("h3".parse().unwrap()),
                Move("Nb8".parse().unwrap()),
                MoveNumberAndPeriods(10, 1),
                Move("d4".parse().unwrap()),
                Move("Nbd7".parse().unwrap()),
                MoveNumberAndPeriods(11, 1),
                Move("c4".parse().unwrap()),
                Move("c6".parse().unwrap()),
                MoveNumberAndPeriods(12, 1),
                Move("cxb5".parse().unwrap()),
                Move("axb5".parse().unwrap()),
                MoveNumberAndPeriods(13, 1),
                Move("Nc3".parse().unwrap()),
                Move("Bb7".parse().unwrap()),
                MoveNumberAndPeriods(14, 1),
                Move("Bg5".parse().unwrap()),
                Move("b4".parse().unwrap()),
                MoveNumberAndPeriods(15, 1),
                Move("Nb1".parse().unwrap()),
                Move("h6".parse().unwrap()),
                MoveNumberAndPeriods(16, 1),
                Move("Bh4".parse().unwrap()),
                Move("c5".parse().unwrap()),
                MoveNumberAndPeriods(17, 1),
                Move("dxe5".parse().unwrap()),
                Move("Nxe4".parse().unwrap()),
                MoveNumberAndPeriods(18, 1),
                Move("Bxe7".parse().unwrap()),
                Move("Qxe7".parse().unwrap()),
                MoveNumberAndPeriods(19, 1),
                Move("exd6".parse().unwrap()),
                Move("Qf6".parse().unwrap()),
                MoveNumberAndPeriods(20, 1),
                Move("Nbd2".parse().unwrap()),
                Move("Nxd6".parse().unwrap()),
                MoveNumberAndPeriods(21, 1),
                Move("Nc4".parse().unwrap()),
                Move("Nxc4".parse().unwrap()),
                MoveNumberAndPeriods(22, 1),
                Move("Bxc4".parse().unwrap()),
                Move("Nb6".parse().unwrap()),
                MoveNumberAndPeriods(23, 1),
                Move("Ne5".parse().unwrap()),
                Move("Rae8".parse().unwrap()),
                MoveNumberAndPeriods(24, 1),
                Move("Bxf7+".parse().unwrap()),
                Move("Rxf7".parse().unwrap()),
                MoveNumberAndPeriods(25, 1),
                Move("Nxf7".parse().unwrap()),
                Move("Rxe1+".parse().unwrap()),
                MoveNumberAndPeriods(26, 1),
                Move("Qxe1".parse().unwrap()),
                Move("Kxf7".parse().unwrap()),
                MoveNumberAndPeriods(27, 1),
                Move("Qe3".parse().unwrap()),
                Move("Qg5".parse().unwrap()),
                MoveNumberAndPeriods(28, 1),
                Move("Qxg5".parse().unwrap()),
                Move("hxg5".parse().unwrap()),
                MoveNumberAndPeriods(29, 1),
                Move("b3".parse().unwrap()),
                Move("Ke6".parse().unwrap()),
                MoveNumberAndPeriods(30, 1),
                Move("a3".parse().unwrap()),
                Move("Kd6".parse().unwrap()),
                MoveNumberAndPeriods(31, 1),
                Move("axb4".parse().unwrap()),
                Move("cxb4".parse().unwrap()),
                MoveNumberAndPeriods(32, 1),
                Move("Ra5".parse().unwrap()),
                Move("Nd5".parse().unwrap()),
                MoveNumberAndPeriods(33, 1),
                Move("f3".parse().unwrap()),
                Move("Bc8".parse().unwrap()),
                MoveNumberAndPeriods(34, 1),
                Move("Kf2".parse().unwrap()),
                Move("Bf5".parse().unwrap()),
                MoveNumberAndPeriods(35, 1),
                Move("Ra7".parse().unwrap()),
                Move("g6".parse().unwrap()),
                MoveNumberAndPeriods(36, 1),
                Move("Ra6+".parse().unwrap()),
                Move("Kc5".parse().unwrap()),
                MoveNumberAndPeriods(37, 1),
                Move("Ke1".parse().unwrap()),
                Move("Nf4".parse().unwrap()),
                MoveNumberAndPeriods(38, 1),
                Move("g3".parse().unwrap()),
                Move("Nxh3".parse().unwrap()),
                MoveNumberAndPeriods(39, 1),
                Move("Kd2".parse().unwrap()),
                Move("Kb5".parse().unwrap()),
                MoveNumberAndPeriods(40, 1),
                Move("Rd6".parse().unwrap()),
                Move("Kc5".parse().unwrap()),
                MoveNumberAndPeriods(41, 1),
                Move("Ra6".parse().unwrap()),
                Move("Nf2".parse().unwrap()),
                MoveNumberAndPeriods(42, 1),
                Move("g4".parse().unwrap()),
                Move("Bd3".parse().unwrap()),
                MoveNumberAndPeriods(43, 1),
                Move("Re6".parse().unwrap()), 
                Result("1/2-1/2".parse().unwrap())
            ]
        )
    }
}