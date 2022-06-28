use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;

#[allow(dead_code)]
#[derive(Debug)]
pub enum TokenType<'a> {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier,
    String(&'a str),
    Number(f32),
    // Keywords.
    And,
    Class,
    Else,
    False,
    Func,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox { had_error: false }
    }

    pub fn run(&self, source: &str) {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        for token in scanner.tokens {
            println!("{:?}", token);
        }
    }
}

#[derive(Debug)]
pub struct Token<'a> {
    token_type: TokenType<'a>,
    lexeme: String,
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenType<'a>, lexeme: String, line: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:?} {} {}", self.token_type, self.lexeme, self.line)
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 0,
        }
    }

    pub fn add_token(&mut self, token_type: TokenType<'a>) {
        let token = Token::new(
            token_type,
            self.source[self.start..self.current].into(),
            self.line,
        );

        self.tokens.push(token);
    }

    pub fn add_literal(&mut self, token_type: TokenType<'a>) {
        let token = Token::new(
            token_type,
            self.source[self.start..self.current].into(),
            self.line,
        );

        self.tokens.push(token);
    }

    pub fn scan_token(&mut self) {
        match self.advance() {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('\n') => {
                self.line += 1;
            },
            Some(' ') | Some('\r') | Some('\t') => (),
            Some('/') => {
                if self.check('/') {
                    while (self.peek() != Some('\n')) && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            Some('!') => {
                if self.check('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            Some('=') => {
                if self.check('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            Some('<') => {
                if self.check('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            Some('>') => {
                if self.check('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            Some('"') => {
				while (self.peek() != Some('"')) && !self.is_at_end() {
					if self.peek() == Some('\n') {
						self.line += 1;
					}
					self.advance();
				}

				if !self.is_at_end() {
					self.advance();
					self.add_token(TokenType::String(
						self.source[self.start + 1..self.current - 1].into(),
					));
				}
            },
            c => {
				// number
				if self.is_digit(c) {
					while self.is_digit(self.peek()) {
						self.advance();
					}

					if self.peek() == Some('.') && self.is_digit(self.peek_next()) {
						self.advance();
					}

					while self.is_digit(self.peek()) {
						self.advance();
					}

					match self.source[self.start..self.current].parse::<f32>() {
						Ok(f) => self.add_token(TokenType::Number(f)),
						Err(_) => exit(65),
					}
				}
				// identifier
				else if self.is_alphanum(c) {
					while self.is_alphanum(self.peek()) {
						self.advance();
					}

					match self.source[self.start..self.current].into() {
						"and" => self.add_token(TokenType::And),
						"class" => self.add_token(TokenType::Class),
						"else" => self.add_token(TokenType::Else),
						"false" => self.add_token(TokenType::False),
						"fun" => self.add_token(TokenType::Func),
						"if" => self.add_token(TokenType::If),
						"nil" => self.add_token(TokenType::Nil),
						"or" => self.add_token(TokenType::Or),
						"print" => self.add_token(TokenType::Print),
						"return" => self.add_token(TokenType::Return),
						"super" => self.add_token(TokenType::Super),
						"this" => self.add_token(TokenType::This),
						"true" => self.add_token(TokenType::True),
						"var" => self.add_token(TokenType::Var),
						"while" => self.add_token(TokenType::While),
						_ => self.add_token(TokenType::Identifier),
					}
				}
				else {
					println!("Unexpected character. {:?}, {}", c, self.line);
				}
			},
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len() as usize
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        return c;
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        } else {
            return self.source.chars().nth(self.current);
        }
    }

	fn peek_next(&self) -> Option<char> {
		if self.current + 1 >= self.source.len() {
			return Some('\0');
		} else {
			return self.source.chars().nth(self.current + 1);
		}
	}

	fn is_digit(&self, val: Option<char>) -> bool {
		match val {
			Some('0') | Some('1') | Some('2') | Some('3') | Some('4') |
			Some('5') | Some('6') | Some('7') | Some('8') | Some('9') => true,
			None => false,
			_ => false,
		}
	}

	fn is_alpha(&self, val: Option<char>) -> bool {
		match val {
			Some(c) => {
				(c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z' || c == '_')
			},
			None => false
		}
	}

	fn is_alphanum(&self, val: Option<char>) -> bool {
		self.is_digit(val) || self.is_alpha(val)
	}

    fn check(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.source.chars().nth(self.current) {
            Some(c) => {
                if c == expected {
                    self.current += 1;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

fn run_file(file_path: &Path) {
    match fs::read_to_string(file_path) {
        Ok(s) => {
            let lox = Lox::new();
            lox.run(&s);
            if lox.had_error {
                exit(65);
            }
        }
        Err(_) => exit(64),
    }
}

fn run_prompt() {
    let mut lox = Lox::new();

    loop {
        let mut buffer = String::new();
        let stdin = io::stdin();
        print!(">> ");
        io::stdout().flush().unwrap();
        match stdin.read_line(&mut buffer) {
            Ok(0) | Err(_) => break,
            _ => (),
        }

        lox.run(&buffer);
        lox.had_error = false;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        run_file(&Path::new(&args[2]));
    } else {
        println!("Usage: jlox [script]");
        exit(64);
    }
}
