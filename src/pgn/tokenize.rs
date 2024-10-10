use std::iter::Peekable;
use std::str::Chars;
use crate::pgn::error::PgnParseError;

/// Represents a token in a PGN string
#[derive(Debug, PartialEq)]
pub enum PgnToken {
    Tag(String),            // Represents a tag (e.g., "[Event "F/S Return Match"]")
    Move(String),           // Represents a move (e.g., "e4", "Nf3#")
    MoveNumber(u16),        // Represents a move number (e.g., "1", "2")
    StartVariation,         // Represents the start of a variation ('(')
    EndVariation,           // Represents the end of a variation (')')
    Comment(String),        // Represents a comment (e.g., "{This is a comment}")
    Annotation(String),     // Represents an annotation (e.g., "!", "?", "!?", etc.)
    Result(String),         // Represents a game result (e.g., "1-0", "0-1", "1/2-1/2")
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
            _ if ch.is_numeric() => {
                // Could be a move number or a result
                let move_number_or_result = collect_until(&mut chars, |c| c == '.' || c.is_ascii_whitespace());
                if move_number_or_result.contains('-') {
                    tokens.push(PgnToken::Result(move_number_or_result));
                }
                else if let Ok(num) = move_number_or_result.parse::<u16>() {
                    tokens.push(PgnToken::MoveNumber(num));
                }
                else {
                    return Err(PgnParseError::InvalidToken(move_number_or_result));
                }
            }
            '.' => {
                // Skip periods
                chars.next();
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
        println!("{:?}", tokens);
    }
}