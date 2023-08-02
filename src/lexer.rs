use peekable_next::{PeekNext, PeekableNext};
use std::str::{CharIndices, FromStr};
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace, Comma,
    Dot, Minus, Plus, Semicolon, Slash, Star,
    
    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    
    // Literals.
    Identifier, String, Number,
    
    // Keywords.
    And, Class, Else, False, For, Fun, If, Nil,
    Or, Print, Return, Super, This, True, Var, While,

    Error,
    Eof,
}

impl FromStr for TokenType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "and" => Ok(Self::And),
            "class" => Ok(Self::Class),
            "else" => Ok(Self::Else),
            "false" => Ok(Self::False),
            "for" => Ok(Self::For),
            "fun" => Ok(Self::Fun),
            "if" => Ok(Self::If),
            "nil" => Ok(Self::Nil),
            "or" => Ok(Self::Or),
            "print" => Ok(Self::Print),
            "return" => Ok(Self::Return),
            "super" => Ok(Self::Super),
            "this" => Ok(Self::This),
            "true" => Ok(Self::True),
            "var" => Ok(Self::Var),
            "while" => Ok(Self::While),
            _ => Ok(Self::Identifier),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Token<'a> {
    lexum: &'a str,
    id: TokenType,
    line: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn new(lexum: &'a str, id: TokenType, line: usize) -> Self {
        Self { lexum, id, line }
    }
}
pub(crate) struct Lexer<'a> {
    source: &'a str,
    start_pos: usize,
    line: usize,
    at_end: bool,
    chars: PeekableNext<CharIndices<'a>>,
}

impl<'a> Lexer<'a>
where
    Self: 'a,
{
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            start_pos: 0,
            at_end: false,
            line: 1,
            chars: source.char_indices().peekable_next(),
        }
    }
}

impl<'a> Iterator for Lexer<'a>
where
    Self: 'a,
{
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }

        let Some((i, ch)) = self.chars.next() else {
	    self.at_end = true;
	    return Some(Token::new("", TokenType::Eof, self.line));
	};

        let token = match ch {
            '(' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::LeftParen,
                self.line,
            )),
            ')' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::RightParen,
                self.line,
            )),
            '{' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::LeftBrace,
                self.line,
            )),
            '}' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::RightBrace,
                self.line,
            )),
            ',' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Comma,
                self.line,
            )),
            '.' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Dot,
                self.line,
            )),
            '-' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Minus,
                self.line,
            )),
            '+' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Plus,
                self.line,
            )),
            ';' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Semicolon,
                self.line,
            )),
            '/' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Slash,
                self.line,
            )),
            '*' => Some(Token::new(
                &self.source[self.start_pos..=i],
                TokenType::Star,
                self.line,
            )),
            _ => todo!(),
        };
        self.start_pos = i + 1;
        token
    }
}
#[cfg(test)]
mod test {
    use super::*;

    macro_rules! make_token {
        ($string:literal, $token:tt, $line:literal) => {
            Token {
                lexum: $string,
                id: TokenType::$token,
                line: $line,
            }
        };
    }

    #[test]
    fn single_character_tokens() {
        let test_str = "(){},.-+/*";
        let token_array = vec![
            make_token!("(", LeftParen, 1),
            make_token!(")", RightParen, 1),
            make_token!("{", LeftBrace, 1),
            make_token!("}", RightBrace, 1),
            make_token!(",", Comma, 1),
            make_token!(".", Dot, 1),
            make_token!("-", Minus, 1),
            make_token!("+", Plus, 1),
            make_token!("/", Slash, 1),
            make_token!("*", Star, 1),
            make_token!("", Eof, 1),
        ];
        let result = Lexer::new(test_str).collect::<Vec<_>>();
        assert_eq!(token_array, result)
    }
}
