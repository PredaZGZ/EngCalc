/// Tokenizer for the engcalc expression language.
///
/// Decision: Using a manual hand-written lexer instead of chumsky/pest because:
/// 1. The language is small and well-defined
/// 2. A manual lexer gives us precise control over error messages
/// 3. No extra dependency overhead
/// 4. Easier to extend with implicit multiplication detection
/// 5. Better integration with our recursive-descent parser
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Equals,
    Comma,
    In,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum LexerError {
    #[error("invalid character '{char}' at position {position}")]
    InvalidCharacter { char: char, position: usize },
    #[error("invalid number at position {position}")]
    InvalidNumber { position: usize },
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, LexerError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let len = chars.len();

    while pos < len {
        let ch = chars[pos];

        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && pos + 1 < len && chars[pos + 1].is_ascii_digit()) {
            let start = pos;
            let mut num_str = String::new();
            while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                num_str.push(chars[pos]);
                pos += 1;
            }
            // Handle scientific notation: 6.022e23, 1e-5, 1E+10
            if pos < len && (chars[pos] == 'e' || chars[pos] == 'E') {
                num_str.push(chars[pos]);
                pos += 1;
                if pos < len && (chars[pos] == '+' || chars[pos] == '-') {
                    num_str.push(chars[pos]);
                    pos += 1;
                }
                let mut has_digit = false;
                while pos < len && chars[pos].is_ascii_digit() {
                    num_str.push(chars[pos]);
                    pos += 1;
                    has_digit = true;
                }
                if !has_digit {
                    return Err(LexerError::InvalidNumber { position: start });
                }
            }
            let number: f64 = num_str
                .parse()
                .map_err(|_| LexerError::InvalidNumber { position: start })?;
            tokens.push(SpannedToken {
                token: Token::Number(number),
                start,
                end: pos,
            });
            continue;
        }

        if ch.is_alphabetic() || ch == '_' || is_greek_char(ch) {
            let start = pos;
            let mut ident = String::new();
            while pos < len && (chars[pos].is_alphanumeric() || chars[pos] == '_' || is_greek_char(chars[pos])) {
                ident.push(chars[pos]);
                pos += 1;
            }
            let token = match ident.as_str() {
                "in" => Token::In,
                _ => Token::Identifier(ident),
            };
            tokens.push(SpannedToken {
                token,
                start,
                end: pos,
            });
            continue;
        }

        let start = pos;
        let token = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '^' => Token::Caret,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '=' => Token::Equals,
            ',' => Token::Comma,
            _ => {
                return Err(LexerError::InvalidCharacter {
                    char: ch,
                    position: pos,
                });
            }
        };
        pos += 1;
        tokens.push(SpannedToken {
            token,
            start,
            end: pos,
        });
    }

    tokens.push(SpannedToken {
        token: Token::Eof,
        start: pos,
        end: pos,
    });

    Ok(tokens)
}

/// Check if a character is a Greek letter (lowercase or uppercase)
fn is_greek_char(ch: char) -> bool {
    matches!(ch,
        'α'..='ω' |  // Greek lowercase
        'Α'..='Ω'    // Greek uppercase
    )
}
