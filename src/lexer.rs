
use std::{iter::Peekable, str::{CharIndices, FromStr}};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Token<'a> {
    pub(crate) lexum: &'a str,
    pub(crate) id: TokenType,
    pub(crate) line: usize,
}

impl<'a> Token<'a> {
    pub(crate) fn new(lexum: &'a str, id: TokenType, line: usize) -> Self {
        Self { lexum, id, line }
    }
}
#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    source: &'a str,
    start_pos: usize,
    line: usize,
    at_end: bool,
    chars: Peekable<CharIndices<'a>>,
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
            chars: source.char_indices().peekable(),
        }
    }
}
impl<'a> Lexer<'a>
where
    Self: 'a,
{
    fn check_token(
        &mut self,
        ch: char,
        pos: usize,
        if_true: TokenType,
        if_not: TokenType,
    ) -> (Option<Token<'a>>, usize) {
        if let Some(a) = self.chars.next_if(|c| c.1 == ch) {
            (
                Some(Token::new(
                    &self.source[self.start_pos..=a.0],
                    if_true,
                    self.line,
                )),
                a.0 + 1,
            )
        } else {
            (
                Some(Token::new(
                    &self.source[self.start_pos..=pos],
                    if_not,
                    self.line,
                )),
                pos + 1,
            )
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
        let mut line = self.line;
        while let Some(_) = self.chars.next_if(|c| {
            if c.1 == '\n' {
                line += 1;
                true
            } else {
                c.1 == ' ' || c.1 == '\r' || c.1 == '\t'
            }
        }) {
            self.start_pos += 1;
        }
        self.line = line;
        let Some((i, ch)) = self.chars.next() else {
	    self.at_end = true;
	    return Some(Token::new("", TokenType::Eof, self.line));
	};

        let token = match ch {
            '(' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::LeftParen,
                    self.line,
                )),
                i + 1,
            ),
            ')' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::RightParen,
                    self.line,
                )),
                i + 1,
            ),
            '{' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::LeftBrace,
                    self.line,
                )),
                i + 1,
            ),
            '}' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::RightBrace,
                    self.line,
                )),
                i + 1,
            ),
            ',' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Comma,
                    self.line,
                )),
                i + 1,
            ),
            '.' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Dot,
                    self.line,
                )),
                i + 1,
            ),
            '-' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Minus,
                    self.line,
                )),
                i + 1,
            ),
            '+' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Plus,
                    self.line,
                )),
                i + 1,
            ),
            ';' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Semicolon,
                    self.line,
                )),
                i + 1,
            ),
            '/' => {
                if self.chars.peek().map(|c| c.1) == Some('/') {
                    while self.chars.next_if(|c| c.1 != '\n' ).is_some() {}
                    // consume the '\n'
                    if let Some((pos, _)) = self.chars.next() {
			self.line += 1;
                        self.start_pos = pos + 1;
                    }
		    return self.next();
                }

                (
                    Some(Token::new(
                        &self.source[self.start_pos..=i],
                        TokenType::Slash,
                        self.line,
                    )),
                    i + 1,
                )
            }
            '*' => (
                Some(Token::new(
                    &self.source[self.start_pos..=i],
                    TokenType::Star,
                    self.line,
                )),
                i + 1,
            ),
            '!' => self.check_token('=', i, TokenType::BangEqual, TokenType::Bang),
            '>' => self.check_token('=', i, TokenType::GreaterEqual, TokenType::Greater),
            '<' => self.check_token('=', i, TokenType::LessEqual, TokenType::Less),
            '=' => self.check_token('=', i, TokenType::EqualEqual, TokenType::Equal),
            '"' => {
                while let Some(_) = self.chars.next_if(|c| c.1 != '"') {}
                if let Some((i, '"')) = self.chars.next() {
                    (
                        Some(Token::new(
                            &self.source[self.start_pos..=i],
                            TokenType::String,
                            self.line,
                        )),
                        i + 1,
                    )
                } else {
                    todo!("handle unterminated strings")
                }
            }
            c if c.is_ascii_digit() => {
let mut i = self.start_pos;
                while let Some((s, _)) = self.chars.next_if(|c| c.1.is_ascii_digit()) {
                    i +=1;
                }
                if let Some((_, '.')) = self.chars.next_if(|c| c.1 == '.') {
		    i += 1;
                    while let Some((s, _)) = self.chars.next_if(|c| c.1.is_ascii_digit()) {

			i += 1;
                    }
                }
                (
                    Some(Token::new(
                        &self.source[self.start_pos..=i],
                        TokenType::Number,
                        self.line,
                    )),
                    i + 1,
                )
            }
            c if c.is_ascii_alphabetic() => {
                let mut i = self.start_pos;
                while let Some(_) = self.chars.next_if(|&(_, ch)| {
                    ch >= 'a' && ch <= 'z' || ch >= 'A' && ch <= 'Z' || ch == '_'
                }) {
                    i += 1;
                }
                let ident = self.source[self.start_pos..=i]
                    .parse::<TokenType>()
                    .unwrap();
                (
                    Some(Token::new(
                        &self.source[self.start_pos..=i],
                        ident,
                        self.line,
                    )),
                    i + 1,
                )
            }
            _ => todo!(),
        };
        self.start_pos = token.1;
        token.0
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
    #[test]
    fn two_character_tokens() {
        let test_str = "!!=>>=<<====";
        let token_array = vec![
            make_token!("!", Bang, 1),
            make_token!("!=", BangEqual, 1),
            make_token!(">", Greater, 1),
            make_token!(">=", GreaterEqual, 1),
            make_token!("<", Less, 1),
            make_token!("<=", LessEqual, 1),
            make_token!("==", EqualEqual, 1),
            make_token!("=", Equal, 1),
            make_token!("", Eof, 1),
        ];
        let result = Lexer::new(test_str).collect::<Vec<_>>();
        assert_eq!(token_array, result)
    }
    #[test]
    fn strings() {
        let test_str = "\"hello\"";
        let token = make_token!("\"hello\"", String, 1);
        let mut lexer = Lexer::new(test_str);
        assert_eq!(Some(token), lexer.next());
    }
    #[test]
    fn num_no_decimal() {
        let test_str = "123";
        let token = make_token!("123", Number, 1);
        let mut lexer = Lexer::new(test_str);
        assert_eq!(Some(token), lexer.next())
    }
    #[test]
    fn num_with_decimal() {
        let test_str = "123.456";
        let token = make_token!("123.456", Number, 1);
        let mut lexer = Lexer::new(test_str);
        assert_eq!(Some(token), lexer.next())
    }
    #[test]
    fn whitespace() {
        let test_str = "/ =";
        let tokens = vec![
            make_token!("/", Slash, 1),
            make_token!("=", Equal, 1),
            make_token!("", Eof, 1),
        ];
        let t = Lexer::new(test_str).collect::<Vec<_>>();
        assert_eq!(tokens, t);
    }
    #[test]
    fn identifiers() {
        let test_str = "fun class me";
        let expected = make_token!("fun", Fun, 1);
        let mut lexer = Lexer::new(test_str);
        assert_eq!(Some(expected), lexer.next());
        assert_eq!(Some(make_token!("class", Class, 1)), lexer.next());
        assert_eq!(Some(make_token!("me", Identifier, 1)), lexer.next());
    }
    #[test]
    fn comments() {
	let test_str = "// hello mom \nfun class me";
        let expected = make_token!("fun", Fun, 2);
        let mut lexer = Lexer::new(test_str);
        assert_eq!(Some(expected), lexer.next());
        assert_eq!(Some(make_token!("class", Class, 2)), lexer.next());
        assert_eq!(Some(make_token!("me", Identifier, 2)), lexer.next());
    }
    
}
