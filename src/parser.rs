use std::iter::Peekable;

use crate::tokenizer::{OperationKind, Span, SpecialKind, Token, TokenKind, Tokenizer};

/// Binary Operation.
#[derive(Debug, PartialEq)]
pub enum BinaryOperation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

/// Unary operation.
#[derive(Debug, PartialEq)]
pub enum UnaryOperation {
    Negation,
}

/// Arithmetic expression.
/// This is the root of our syntax tree.
#[derive(Debug, PartialEq)]
pub enum Expression {
    /// Binary expression.
    Binary {
        operation: BinaryOperation,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    /// Unary expression.
    Unary {
        operation: UnaryOperation,
        operand: Box<Expression>,
    },
    /// Atom, in this case a number.
    Atom(f64),
}

#[derive(Debug, PartialEq)]
pub enum ParseTree {
    /// A parsed arithmetic expression.
    Expression(Expression),
    /// A quit instruction.
    Quit,
    /// Nothing to parse.
    Empty,
}

/// An error catched by the parser.
pub enum ParserError {
    /// The error occured because the special command was not recognized.
    UnrecognizedSpecial(Option<Span>),
    /// The error occured because the parser expected a binary operator
    /// (`+`,`-`,`*` or `/`) but got something else instead.
    ExpectedBinaryOp(Option<Span>),
    /// The error occured because the parser expected a new expression
    /// (`-`, `(`, or a number), but got something else instead.
    ExpectedExprStart(Option<Span>),
    /// The error occured because the parser expected a closing parenthesis
    /// but got something else instead.
    UnclosedParenthesis(Option<Span>),
}

/// Parser datastructure.
pub struct Parser<'a> {
    /// Tokenizer.
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from source input.
    pub fn new(input: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }

    /// Entrypoint for parsing.
    pub fn parse(self) -> Result<ParseTree, ParserError> {
        let mut tokens = self.tokenizer.tokenize();
        let parse_tree = match tokens.peek() {
            // If there are not tokens to parse, return an empty parse tree.
            None => Ok(ParseTree::Empty),
            // If the first token is a special token, handle it.
            Some(token) if token.kind == TokenKind::Special(SpecialKind::Quit) => {
                Ok(ParseTree::Quit)
            }
            Some(token) if token.kind == TokenKind::Special(SpecialKind::Unrecognized) => {
                Err(ParserError::UnrecognizedSpecial(Some(token.span)))
            }
            // Otherwise, parse the tokens using a pratt parser.
            _ => Ok(ParseTree::Expression(Self::pratt_parser(&mut tokens, 0)?)),
        };

        parse_tree
    }

    /// Describes the binding power of unary operators.
    fn prefix_binding_power(op: &UnaryOperation) -> u8 {
        match op {
            UnaryOperation::Negation => 5,
        }
    }

    /// Describes the binding power of infix operators.
    fn infix_binding_power(op: &BinaryOperation) -> (u8, u8) {
        match op {
            BinaryOperation::Addition | BinaryOperation::Subtraction => (1, 2),
            BinaryOperation::Multiplication | BinaryOperation::Division => (3, 4),
        }
    }

    /// A priority parser using the Pratt algorithm.
    /// This is the main parsing function.
    fn pratt_parser(
        tokens: &mut Peekable<impl Iterator<Item = Token>>,
        min_bp: u8,
    ) -> Result<Expression, ParserError> {
        // Handles tokens that can start an expression
        let mut lhs = match tokens.next() {
            // Numbers
            Some(Token {
                kind: TokenKind::Number(num),
                ..
            }) => Expression::Atom(num),
            // Unary operators
            Some(Token {
                kind: TokenKind::Operation(OperationKind::Minus),
                ..
            }) => {
                let op = UnaryOperation::Negation;
                // Recursive pratt parser call
                let rhs = Self::pratt_parser(tokens, Self::prefix_binding_power(&op))?;
                Expression::Unary {
                    operation: op,
                    operand: Box::new(rhs),
                }
            }
            // Parenthesis
            Some(Token {
                kind: TokenKind::OpenParenthesis,
                ..
            }) => {
                // Recursive pratt parser call
                let lhs = Self::pratt_parser(tokens, 0)?;
                // Consume the closing parenthesis
                let closing_parenthesis = tokens.next();
                // Check if parenthesis is matched
                if !matches!(
                    closing_parenthesis,
                    Some(Token {
                        kind: TokenKind::CloseParenthesis,
                        ..
                    })
                ) {
                    return Err(ParserError::UnclosedParenthesis(
                        closing_parenthesis.map(|token| token.span),
                    ));
                }

                lhs
            }
            t => return Err(ParserError::ExpectedExprStart(t.map(|token| token.span))),
        };

        loop {
            let op = match tokens.peek() {
                // Break if end of input is reached.
                None => break,
                // Break if a closing parenthesis is reached.
                Some(Token {
                    kind: TokenKind::CloseParenthesis,
                    ..
                }) => break,

                // Transform tokens into `BinaryOperation`s.
                Some(Token {
                    kind: TokenKind::Operation(op),
                    ..
                }) => match op {
                    OperationKind::Plus => BinaryOperation::Addition,
                    OperationKind::Minus => BinaryOperation::Subtraction,
                    OperationKind::Star => BinaryOperation::Multiplication,
                    OperationKind::Slash => BinaryOperation::Division,
                },

                t => return Err(ParserError::ExpectedBinaryOp(t.map(|token| token.span))),
            };

            // Handle binding powers
            let (l_bp, r_bp) = Self::infix_binding_power(&op);
            if l_bp < min_bp {
                break;
            }

            // Consume the operation token
            tokens.next();

            // Recursive pratt parser call
            let rhs = Self::pratt_parser(tokens, r_bp)?;

            lhs = Expression::Binary {
                operation: op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }
}
