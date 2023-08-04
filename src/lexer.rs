use std::{
    iter::Peekable,
    ops::{ControlFlow, RangeInclusive},
    str::{CharIndices, FromStr},
};
#[derive(Debug, PartialEq, Eq)]
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
            _ if s.chars().try_for_each(|c| match c {
                '0'..='9' => ControlFlow::Continue(()),
                '_' => ControlFlow::Continue(()),
                _ => ControlFlow::Break(()),
            }) == ControlFlow::Continue(()) =>
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

impl<'a> Token<'a> {
    pub(crate) fn new(id: TokenType, lexum: &'a str, line: usize) -> Self {
        Self { id, lexum, line }
    }
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    start_pos: usize,
    line: usize,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            source,
            start_pos: 0,
            line: 1,
            chars: source.char_indices().peekable(),
        }
    }
    #[inline(always)]
    fn at_end(&mut self) -> bool {
        self.chars.peek().is_none()
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
        let mut line = self.line;
        let ControlFlow::Break(((cur_pos, ch), line)) = self.chars.try_for_each(move |x| {
	    match x.1 {
		' ' | '\r' | '\t' => ControlFlow::Continue(()),
		'\n' => {
		    line += 1;
		    ControlFlow::Continue(())
		}
		_ => ControlFlow::Break((x, line)),
	    }
        }) else {
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
	    '!' | '>' | '<' | '=' => Token::new(
		self.source[self.get_range(cur_pos)].parse().unwrap(),
		&self.source[self.get_range(cur_pos)],
		self.line
	    ),
            _ => todo!(),
        };
        Some(Ok(token))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_char_token() {
        let source = "() {} , . - + ; *";
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
}
