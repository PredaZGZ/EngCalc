/// Recursive-descent parser for engcalc.
///
/// Grammar (precedence low to high):
///   expr          -> function_def | assignment | unit_convert
///   function_def  -> IDENTIFIER "(" params ")" "=" expr
///   params        -> IDENTIFIER ("," IDENTIFIER)*
///   assignment    -> IDENTIFIER "=" expr
///   unit_convert  -> addition_sub "in" compound_unit
///   addition_sub  -> mul_div_mod (("+"|"-") mul_div_mod)*
///   mul_div_mod   -> unary (("*"|"/"|"%") unary | implicit_mul)*
///   unary         -> ("-" unary) | pow
///   pow           -> call ("^" unary)?
///   call          -> primary ("(" args? ")")?
///   primary       -> NUMBER | IDENTIFIER | "(" expr ")"
///   args          -> expr ("," expr)*
///   compound_unit -> unit ("/"|"*" unit)*
///   unit          -> IDENTIFIER ("^" NUMBER)?
///
/// Implicit multiplication is detected between:
///   - NUMBER followed by IDENTIFIER or LPAREN
///   - RPAREN followed by IDENTIFIER or LPAREN
///
/// Key decisions:
/// - Power is right-associative: 2^3^2 = 2^(3^2) = 512
/// - Unary minus binds tighter than power: -2^2 = -(2^2) = -4
use crate::core::ast::*;
use crate::core::lexer::*;
use crate::core::units;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected end of expression")]
    UnexpectedEof,
    #[error("expected identifier")]
    ExpectedIdentifier,
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unexpected token '{token}'")]
    UnexpectedTokenDetail { token: String },
    #[error("unmatched parenthesis")]
    UnmatchedParen,
    #[error("expected ')' after function arguments")]
    ExpectedCloseParen,
    #[error("incomplete expression")]
    Incomplete,
}

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos].token
        } else {
            &Token::Eof
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let idx = self.pos + offset;
        if idx < self.tokens.len() {
            &self.tokens[idx].token
        } else {
            &Token::Eof
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let got = self.advance();
        if got == expected {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken)
        }
    }

    fn at_end(&self) -> bool {
        matches!(self.current(), Token::Eof)
    }

    fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_expr()?;
        if !self.at_end() {
            return Err(ParseError::UnexpectedTokenDetail {
                token: format!("{:?}", self.current()),
            });
        }
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Check for function definition: IDENTIFIER '(' params ')' '=' expr
        if let Token::Identifier(name) = self.current().clone() {
            let saved = self.pos;
            self.advance();
            if matches!(self.current(), Token::LParen) {
                // Try to parse as function definition
                self.advance();
                // Try to parse params - if this fails, it's probably a function call, not a def
                match self.parse_params() {
                    Ok(params) => {
                        if self.expect(Token::RParen).is_ok()
                            && matches!(self.current(), Token::Equals)
                        {
                            self.advance();
                            let body = self.parse_assignment_or_convert()?;
                            return Ok(Expr::FunctionDef {
                                name,
                                params,
                                body: Box::new(body),
                            });
                        }
                    }
                    Err(_) => {
                        // Not a valid function definition, restore and try as function call
                    }
                }
                // Not a function def, restore and continue
                self.pos = saved;
            } else if matches!(self.current(), Token::Equals) {
                // Assignment: IDENTIFIER '=' expr
                self.advance();
                let value = self.parse_assignment_or_convert()?;
                return Ok(Expr::Assignment {
                    name,
                    value: Box::new(value),
                });
            }
            self.pos = saved;
        }
        self.parse_assignment_or_convert()
    }

    fn parse_params(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();

        // Empty params
        if matches!(self.current(), Token::RParen) {
            return Ok(params);
        }

        // First param
        match self.current().clone() {
            Token::Identifier(name) => {
                params.push(name);
                self.advance();
            }
            _ => return Err(ParseError::ExpectedIdentifier),
        }

        // Additional params
        while matches!(self.current(), Token::Comma) {
            self.advance();
            match self.current().clone() {
                Token::Identifier(name) => {
                    params.push(name);
                    self.advance();
                }
                _ => return Err(ParseError::ExpectedIdentifier),
            }
        }

        Ok(params)
    }

    fn parse_assignment_or_convert(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add_sub()?;

        if let Token::In = self.current() {
            self.advance();
            // Parse compound unit string after "in"
            let unit = self.parse_compound_unit_string();
            if let Some(unit_str) = unit {
                return Ok(Expr::UnitConvert {
                    value: Box::new(left),
                    target_unit: unit_str,
                });
            }
            return Err(ParseError::ExpectedIdentifier);
        }

        Ok(left)
    }

    fn parse_add_sub(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_div_mod()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_div_mod()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_mul_div_mod(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            // Check for explicit operators first
            let op = match self.current() {
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                Token::Percent => BinaryOperator::Mod,
                _ => {
                    // Check for implicit multiplication
                    if self.check_implicit_mul() {
                        BinaryOperator::Mul
                    } else {
                        break;
                    }
                }
            };

            // For implicit multiplication, we don't consume a token
            if matches!(self.current(), Token::Star | Token::Slash | Token::Percent) {
                self.advance();
            }

            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Check if current token position implies multiplication with the previous expression.
    fn check_implicit_mul(&self) -> bool {
        match self.current() {
            // After a parenthesized expression, implicit mul with: ident or (
            Token::LParen => true,
            // Number followed by ident (for unit values or variables like 2x)
            Token::Identifier(_) => true,
            _ => false,
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.current(), Token::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp {
                op: UnaryOperator::Neg,
                operand: Box::new(operand),
            });
        }
        self.parse_pow()
    }

    fn parse_pow(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_call()?;

        if matches!(self.current(), Token::Caret) {
            self.advance();
            let exponent = self.parse_unary()?;
            return Ok(Expr::BinaryOp {
                op: BinaryOperator::Pow,
                left: Box::new(base),
                right: Box::new(exponent),
            });
        }

        Ok(base)
    }

    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        match self.parse_primary()? {
            Expr::Identifier(name) => {
                if matches!(self.current(), Token::LParen) {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expr::FunctionCall { name, args });
                }
                Ok(Expr::Identifier(name))
            }
            expr => Ok(expr),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        if matches!(self.current(), Token::RParen) {
            return Ok(vec![]);
        }

        let mut args = vec![self.parse_expr()?];
        while matches!(self.current(), Token::Comma) {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn is_known_unit(name: &str) -> bool {
        name != "in" && units::is_valid_unit(name)
    }

    /// Parse a compound unit like "km/h" or "m*s^2" or "kg*m/s^2"
    /// Only consumes unit tokens, stops when it sees non-unit tokens
    fn parse_compound_unit_string(&mut self) -> Option<String> {
        let mut result = String::new();
        let mut first = true;
        let mut expect_unit = true; // We expect a unit name next

        loop {
            if expect_unit {
                // Expect an identifier (unit name)
                if let Token::Identifier(name) = self.current().clone() {
                    if !Self::is_known_unit(&name) {
                        break;
                    }
                    if !first {
                        result.push('*');
                    }
                    result.push_str(&name);
                    self.advance();
                    first = false;
                    expect_unit = false; // Now expect operator or end

                    // Check for power
                    if matches!(self.current(), Token::Caret) {
                        self.advance();
                        if let Token::Number(n) = self.current().clone() {
                            if n.fract() == 0.0 && n >= -9.0 && n <= 9.0 {
                                result.push('^');
                                result.push_str(&format!("{}", n as i8));
                                self.advance();
                            }
                        }
                    }
                } else {
                    break;
                }
            } else {
                // Expect / or * operator for compound units
                match self.current() {
                    Token::Slash => {
                        result.push('/');
                        self.advance();
                        expect_unit = true; // Expect unit after /
                    }
                    Token::Star => {
                        // Check if next is a unit - if not, this * is for multiplication
                        if let Token::Identifier(name) = self.peek(1).clone() {
                            if Self::is_known_unit(&name) {
                                result.push('*');
                                self.advance();
                                expect_unit = true;
                            } else {
                                break; // * followed by non-unit, stop here
                            }
                        } else {
                            break; // * not followed by identifier, stop here
                        }
                    }
                    _ => break,
                }
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                // Check for compound unit after number
                if let Some(unit_str) = self.parse_compound_unit_string() {
                    return Ok(Expr::UnitValue {
                        value: Box::new(Expr::Number(n)),
                        unit: unit_str,
                    });
                }
                Ok(Expr::Number(n))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Identifier(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)
                    .map_err(|_| ParseError::UnmatchedParen)?;

                // Check for compound unit after paren
                if let Some(unit_str) = self.parse_compound_unit_string() {
                    return Ok(Expr::UnitValue {
                        value: Box::new(expr),
                        unit: unit_str,
                    });
                }

                Ok(expr)
            }
            Token::Eof => Err(ParseError::UnexpectedEof),
            _ => Err(ParseError::UnexpectedTokenDetail {
                token: format!("{:?}", self.current()),
            }),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = crate::core::lexer::tokenize(input).map_err(|e| match e {
        LexerError::InvalidCharacter { position, .. } => ParseError::UnexpectedTokenDetail {
            token: format!("invalid char at {}", position),
        },
        LexerError::InvalidNumber { position } => ParseError::UnexpectedTokenDetail {
            token: format!("invalid number at {}", position),
        },
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
