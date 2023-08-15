use std::{
    fmt::Debug,
    iter::Peekable,
    ops::{ControlFlow, RangeInclusive},
    str::{CharIndices, FromStr},
};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorToken {
    pub message: String,
    pub line: usize,
}

impl ErrorToken {
    pub fn new(message: impl ToString, line: usize) -> Self {
        Self {
            message: message.to_string(),
            line,
        }
    }
}
pub(crate) type LexerResult<'a> = Result<Token<'a>, ErrorToken>;
#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenType{
    LeftParen,  RightParen,
    LeftBrace,  RightBrace,
    Comma, Dot, Minus,  Plus,
    Semicolon, Slash,  Star,
    // One or two character tokens.
    Bang,  BangEqual,
    Equal,  EqualEqual,
    Greater,  GreaterEqual,
    Less,  LessEqual,
    // Literals.
    Identifier,  String,  Number,
    // Keywords.
    And,  Class,  Else,  False,
    For,  Fun,  If,  Nil,  Or,
    Print,  Return,  Super,  This,
    True,  Var,  While,

    None
}

impl FromStr for TokenType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "(" => Self::LeftParen,
            ")" => Self::RightParen,
            "{" => Self::LeftBrace,
            "}" => Self::RightBrace,
            "," => Self::Comma,
            "." => Self::Dot,
            "-" => Self::Minus,
            "+" => Self::Plus,
            ";" => Self::Semicolon,
            "/" => Self::Slash,
            "*" => Self::Star,
            "!" => Self::Bang,
            "!=" => Self::BangEqual,
            "=" => Self::Equal,
            "==" => Self::EqualEqual,
            ">" => Self::Greater,
            ">=" => Self::GreaterEqual,
            "<" => Self::Less,
            "<=" => Self::LessEqual,
            "and" => Self::And,
            "class" => Self::Class,
            "else" => Self::Else,
            "false" => Self::False,
            "for" => Self::For,
            "fun" => Self::Fun,
            "if" => Self::If,
            "nil" => Self::Nil,
            "or" => Self::Or,
            "print" => Self::Print,
            "return" => Self::Return,
            "super" => Self::Super,
            "this" => Self::This,
            "true" => Self::True,
            "var" => Self::Var,
            "while" => Self::While,
            _ if s.chars().peekable().next_if_eq(&'"').is_some()
                && s.chars().last() == Some('"') =>
            {
                Self::String
            }
            _ if s
                .chars()
                .try_for_each(|c| match c {
                    '0'..='9' => ControlFlow::Continue(()),
                    '.' => ControlFlow::Continue(()),
                    _ => ControlFlow::Break(()),
                })
                .is_continue() =>
            {
                Self::Number
            }

            _ => Self::Identifier,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    pub(crate) id: TokenType,
    pub(crate) lexum: &'a str,
    pub(crate) line: usize,
}

impl<'a> Default for Token<'a> {
    fn default() -> Self {
        Token::new(TokenType::None, "", 0)
    }
}

impl<'a> Token<'a> {
    pub(crate) fn new(id: TokenType, lexum: &'a str, line: usize) -> Self {
        Self { id, lexum, line }
    }
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    start_pos: usize,
    at_end: bool,
    pub(crate) line: usize,
    chars: Peekable<CharIndices<'a>>,
}
impl Debug for Lexer<'_> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            start_pos: 0,
            line: 1,
            at_end: false,
            chars: source.char_indices().peekable(),
        }
    }
    #[inline(always)]
    fn get_range(&self, top: usize) -> RangeInclusive<usize> {
        RangeInclusive::new(self.start_pos, top)
    }
}
impl<'a> Iterator for Lexer<'a>
where
    Self: 'a,
{
    type Item = LexerResult<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }
        let mut line = self.line;
        while Some(true)
            == self
                .chars
                .peek()
                .map(|c| match c.1 {
                    ' ' | '\r' | '\t' => Some(true),
                    '\n' => {
                        line += 1;
                        Some(true)
                    }
                    _ => Some(false),
                })
                .flatten()
        {
            self.chars.next();
        }

	let Some((cur_pos, ch)) = self.chars.next() else {
	    return None;
	};
        self.line = line;
        self.start_pos = cur_pos;
        let token = match ch {
            '(' | ')' | '{' | '}' | ',' | '.' | '-' | '+' | ';' | '*' => Token::new(
                self.source[self.get_range(cur_pos)].parse().unwrap(),
                &self.source[self.get_range(cur_pos)],
                self.line,
            ),
            '!' if self.chars.next_if(|x| x.1 == '=').is_some() => {
                let range = self.get_range(cur_pos + 1);
                self.start_pos += 1;
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            '>' if self.chars.next_if(|x| x.1 == '=').is_some() => {
                let range = self.get_range(cur_pos + 1);
                self.start_pos += 1;
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            '<' if self.chars.next_if(|x| x.1 == '=').is_some() => {
                let range = self.get_range(cur_pos + 1);
                self.start_pos += 1;
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            '=' if self.chars.next_if(|x| x.1 == '=').is_some() => {
                let range = self.get_range(cur_pos + 1);
                self.start_pos += 1;
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            '/' if self.chars.next_if(|x| x.1 == '/').is_some() => {
                let ControlFlow::Break(pos) = self.chars.try_for_each(|x| match x.1 {
		    '\n' => ControlFlow::Break(x.0),
		    _ => ControlFlow::Continue(()),
		}) else {
		    return None;
		};
                self.line += 1;
                self.start_pos = pos + 1;
                return self.next();
            }
            '!' | '>' | '<' | '=' | '/' => Token::new(
                self.source[self.get_range(cur_pos)].parse().unwrap(),
                &self.source[self.get_range(cur_pos)],
                self.line,
            ),
            '0'..='9' => {

		let mut pos = self.start_pos;
		while Some(true) == self.chars.peek().map(|ch| match ch.1 {
		    '0'..='9' | '.' => Some(true),
		    _ => Some(false),
		}).flatten() {
		    self.chars.next();
		    pos += 1;
		}
                let range = self.get_range(pos);
		self.start_pos = pos;
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            'a'..='z' | 'A'..='Z' => {
                let mut s = self.start_pos;
                while Some(true) == self.chars.peek().map(|x| match x.1 {
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
			Some(true)
                    }
                    _ => Some(false),
                }).flatten() {
		    s += 1;
		    self.chars.next();
                }
                let range = self.get_range(s);
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            '"' => {
                let mut line = self.line;
                let pos = match self.chars.try_for_each(|x| match x.1 {
                    '"' => ControlFlow::Break(x.0),
                    '\n' => {
                        line += 1;
                        ControlFlow::Continue(())
                    }
                    _ => ControlFlow::Continue(()),
                }) {
                    ControlFlow::Break(x) => x,
                    ControlFlow::Continue(()) => {
                        // We have an unterminated string.
                        return Some(Err(ErrorToken::new("Unterminated string.", self.line)));
                    }
                };
                self.line = line;
                let range = self.get_range(pos);
                Token::new(
                    self.source[range.clone()].parse().unwrap(),
                    &self.source[range],
                    self.line,
                )
            }
            _ => unreachable!(),
        };
        Some(Ok(token))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_char_token() {
        let source = "() {} , . - + ; * / ";
        let expected: Vec<Result<Token<'_>, ErrorToken>> = vec![
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::RightBrace, "}", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::Minus, "-", 1),
            Token::new(TokenType::Plus, "+", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Star, "*", 1),
            Token::new(TokenType::Slash, "/", 1),
        ]
        .into_iter()
        .map(Ok)
        .collect();
        let lexer = Lexer::new(source);
        assert_eq!(expected, lexer.collect::<Vec<_>>());
    }
    #[test]
    fn multi_char_token() {
        let source = "!!=>>==<<===";
        let expected: Vec<Result<Token<'_>, ErrorToken>> = vec![
            Token::new(TokenType::Bang, "!", 1),
            Token::new(TokenType::BangEqual, "!=", 1),
            Token::new(TokenType::Greater, ">", 1),
            Token::new(TokenType::GreaterEqual, ">=", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::Less, "<", 1),
            Token::new(TokenType::LessEqual, "<=", 1),
            Token::new(TokenType::EqualEqual, "==", 1),
        ]
        .into_iter()
        .map(Ok)
        .collect();
        let lexer = Lexer::new(source);
        assert_eq!(expected, lexer.collect::<Vec<_>>());
    }
    #[test]
    fn number() {
        let input = "123 123.456";
        let test_results = ["123", "123.456"]
            .into_iter()
            .map(|x| Token::new(TokenType::Number, x, 1))
            .map(Ok)
            .collect::<Vec<Result<Token, ErrorToken>>>();
        let lexer = Lexer::new(input);
        for (expected, got) in test_results.into_iter().zip(lexer) {
            assert_eq!(expected, got);
        }
    }
    #[test]
    fn comments() {
        let input = "// this is a comment\n*";
        let mut lexer = Lexer::new(input);
        assert_eq!(Some(Ok(Token::new(TokenType::Star, "*", 2))), lexer.next())
    }
    #[test]
    fn identifiers() {
        let input =
            "and class else false for fun if nil or print return super this true var while me";
        let expexted_token = [
            TokenType::And,
            TokenType::Class,
            TokenType::Else,
            TokenType::False,
            TokenType::For,
            TokenType::Fun,
            TokenType::If,
            TokenType::Nil,
            TokenType::Or,
            TokenType::Print,
            TokenType::Return,
            TokenType::Super,
            TokenType::This,
            TokenType::True,
            TokenType::Var,
            TokenType::While,
            TokenType::Identifier,
        ];
        let expected = input
            .split_whitespace()
            .zip(expexted_token.into_iter())
            .map(|x| Token::new(x.1, x.0, 1))
            .map(Ok)
            .collect::<Vec<LexerResult>>();
        let lexer = Lexer::new(input);
        for (expected, got) in expected.into_iter().zip(lexer) {
            assert_eq!(expected, got);
        }
    }
    #[test]
    fn string() {
        let input = "\"hello\"";
        let expected = Token::new(TokenType::String, "\"hello\"", 1);

        assert_eq!(Some(Ok(expected)), Lexer::new(input).next());
    }
}
