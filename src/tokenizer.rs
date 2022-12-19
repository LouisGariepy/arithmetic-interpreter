//! The tokenizer uses a `Cursor` to iterate

use std::{
    iter::Peekable,
    ops::{Index, Range},
    str::Chars,
};

use unicode_xid::UnicodeXID;

/// A cursor over the input characters. The cursor's job
/// is to record the byte position of the characters so
/// we can give our tokens a span.
struct Cursor<'a> {
    /// The character iterator.
    chars: Chars<'a>,
    /// The current byte position of the iterator.
    byte_pos: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor from an input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
            byte_pos: 0,
        }
    }

    /// Peeks the next character *without advancing the character iterator*.
    pub fn peek(&self) -> Option<char> {
        // Cloning `chars` is cheap.
        self.chars.clone().next()
    }

    /// Advances to the next character.
    pub fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        self.byte_pos += c.map(|c| c.len_utf8()).unwrap_or_default();
        c
    }

    /// Advances the cursor while the iterator still has items
    /// and while predicate is `true`.
    pub fn skip_while(&mut self, predicate: fn(char) -> bool) {
        while matches!(self.peek(), Some(c) if predicate(c)) {
            self.next();
        }
    }
}

/// A span that describes the byte position of a token in the source input.
/// Useful to report nice errors.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Span {
    /// Start byte position.
    pub start: usize,
    /// End byte position.
    pub end: usize,
}

/// Allows us to create spans from ranges.
impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Span {
            start: value.start,
            end: value.end,
        }
    }
}

/// Allows us to index strings with our spans.
impl Index<Span> for str {
    type Output = Self;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

/// A token kind for special tokens
#[derive(Debug, PartialEq)]
pub enum SpecialKind {
    /// The quit instruction. We'll use this to let the
    /// user exit the calculator.
    Quit,
    /// An unrecognized special command.
    Unrecognized,
}

/// A token kind for arithmetic operations.
#[derive(Debug, PartialEq)]
pub enum OperationKind {
    /// `+`.
    Plus,
    /// `-`.
    Minus,
    /// `*`.
    Star,
    /// `/`.
    Slash,
}

/// The kind of our tokens.
#[derive(Debug, PartialEq)]
pub enum TokenKind {
    /// Whitespace tokens like ` `, `\t`, `\n`, `\r`...
    Whitespace,
    /// Special tokens.
    Special(SpecialKind),
    /// Numbers. We'll represent all numbers as f64 internally.
    Number(f64),
    /// Symbols for arithmetic operations.
    Operation(OperationKind),
    /// `(`.
    OpenParenthesis,
    /// `)`.
    CloseParenthesis,

    /// Unrecognized tokens.
    Unrecognized,
}

/// Data structure for our tokens.
#[derive(Debug, PartialEq)]
pub struct Token {
    /// The kind of this token.
    pub kind: TokenKind,
    /// The span of this token.
    pub span: Span,
}

/// The tokenizer. Transforms an input string into an iterator of tokens.
pub struct Tokenizer<'a> {
    /// The tokenizer input.
    input: &'a str,
    /// The source cursor.
    cursor: Cursor<'a>,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer from an input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: Cursor::new(input),
        }
    }

    /// Creates a token iterator by calling `next_token` until all the
    /// characters are consumed.
    pub fn tokenize(mut self) -> Peekable<impl Iterator<Item = Token> + 'a> {
        std::iter::from_fn(move || self.next_token())
            .filter(|token| !matches!(token.kind, TokenKind::Whitespace))
            .peekable()
    }

    /// Advances the cursor while the characters are whitespace.
    fn whitespace(&mut self) {
        self.cursor.skip_while(char::is_whitespace);
    }

    /// Advances the cursor while the characters are part of a single identifier.
    fn identifier(&mut self) {
        self.cursor.skip_while(char::is_xid_continue);
    }

    /// Advances the cursor while the characters are part of a single number.
    fn number(&mut self) {
        self.cursor.skip_while(|c: char| c.is_ascii_digit());
        if self.cursor.peek() == Some('.') {
            self.cursor.next(); // Consume the dot
            self.cursor.skip_while(|c: char| c.is_ascii_digit());
        }
    }

    /// Advances the cursor to create the single next token.
    /// This is the main tokenizing function.
    fn next_token(&mut self) -> Option<Token> {
        // Record the start of the token.
        let start = self.cursor.byte_pos;
        // First, we take one single character.
        let c = self.cursor.next();

        // Then, we match that character to find the kind of the token
        let kind = match c {
            // Whitespace token.
            Some(c) if c.is_whitespace() => {
                self.whitespace();
                TokenKind::Whitespace
            }

            // Special token (starts with `?`).
            Some('?') => {
                self.identifier();
                let identifier = &self.input[(start + 1)..self.cursor.byte_pos];
                match identifier {
                    "quit" => TokenKind::Special(SpecialKind::Quit),
                    _ => TokenKind::Special(SpecialKind::Unrecognized),
                }
            }

            // Number token.
            Some(c) if c.is_ascii_digit() => {
                self.number();
                let number = &self.input[start..self.cursor.byte_pos];
                TokenKind::Number(number.parse().unwrap())
            }

            // Operation tokens
            Some('+') => TokenKind::Operation(OperationKind::Plus),
            Some('-') => TokenKind::Operation(OperationKind::Minus),
            Some('*') => TokenKind::Operation(OperationKind::Star),
            Some('/') => TokenKind::Operation(OperationKind::Slash),

            // Parenthesis tokens
            Some('(') => TokenKind::OpenParenthesis,
            Some(')') => TokenKind::CloseParenthesis,

            // Any other character is unrecognized
            Some(_) => TokenKind::Unrecognized,

            // We consumed all the characters
            None => return None,
        };

        // Record the end of the token
        let end = self.cursor.byte_pos;
        // Now we know the span of the token
        let span = Span::from(start..end);

        // Return the token
        Some(Token { kind, span })
    }
}

/// Tests for the tokenizer.
#[cfg(test)]
mod tests {
    use crate::tokenizer::{OperationKind, SpecialKind, Token, TokenKind, Tokenizer};

    #[test]
    fn test_whitespace() {
        let input = " \n\r\t";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: super::TokenKind::Whitespace,
                span: (0..4).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_special_quit() {
        let input = "?quit";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Special(SpecialKind::Quit),
                span: (0..5).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_special_unrecognized() {
        let input = "?blabla";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Special(SpecialKind::Unrecognized),
                span: (0..7).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_number() {
        let input = "123.123";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Number(123.123),
                span: (0..7).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_number_trailing_dot() {
        let input = "123.";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Number(123.),
                span: (0..4).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_number_no_decimal() {
        let input = "123";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Number(123.),
                span: (0..3).into()
            }],
            tokens
        );
    }

    #[test]
    fn test_plus() {
        let input = "+";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Operation(OperationKind::Plus),
                span: (0..1).into()
            }],
            tokens
        );
    }
    #[test]
    fn test_minus() {
        let input = "-";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Operation(OperationKind::Minus),
                span: (0..1).into()
            }],
            tokens
        );
    }
    #[test]
    fn test_star() {
        let input = "*";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Operation(OperationKind::Star),
                span: (0..1).into()
            }],
            tokens
        );
    }
    #[test]
    fn test_slash() {
        let input = "/";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::Operation(OperationKind::Slash),
                span: (0..1).into()
            }],
            tokens
        );
    }
    #[test]
    fn test_open_parenthesis() {
        let input = "(";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::OpenParenthesis,
                span: (0..1).into()
            }],
            tokens
        );
    }
    #[test]
    fn test_close_parenthesis() {
        let input = ")";
        let tokens = Tokenizer::new(input).tokenize().collect::<Vec<_>>();
        assert_eq!(
            vec![Token {
                kind: TokenKind::CloseParenthesis,
                span: (0..1).into()
            }],
            tokens
        );
    }
}
