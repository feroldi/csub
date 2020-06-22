#![allow(dead_code)]

use std::{iter::Peekable, str::Chars};

use crate::source_map::{BytePos, Pos, Span};

#[derive(Debug, PartialEq)]
pub enum Category {
    Kw(Keyword),
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    ExclamaEqual,
    Equal,
    Semi,
    Comma,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Ident,  // letter (letter | digit)*
    Number, // digit digit*
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Else,
    If,
    Int,
    Return,
    Void,
    While,
}

pub struct Word {
    pub category: Category,
    pub lexeme: Span,
}

pub trait Scanner {
    fn scan_next_word(&mut self) -> Option<Word>;
}

struct CharBumper<'chars> {
    char_stream: Peekable<Chars<'chars>>,
    cur_peek_pos: BytePos,
}

impl<'chars> CharBumper<'chars> {
    fn new(chars: Chars<'chars>) -> CharBumper<'chars> {
        CharBumper {
            char_stream: chars.peekable(),
            cur_peek_pos: BytePos(0),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.char_stream.peek().map(|&c| c)
    }

    fn peek_is(&mut self, ch: char) -> bool {
        self.peek() == Some(ch)
    }

    fn bump(&mut self) -> Option<char> {
        let next_char = self.char_stream.next();
        next_char.and_then(|c| {
            let byte_length_in_utf8 = Pos::from_usize(c.len_utf8());
            self.cur_peek_pos = self.cur_peek_pos + byte_length_in_utf8;
            Some(c)
        })
    }
}

struct CSubScanner<'chars> {
    char_stream: CharBumper<'chars>,
}

impl CSubScanner<'_> {
    fn with_chars(chars: Chars<'_>) -> CSubScanner<'_> {
        CSubScanner {
            char_stream: CharBumper::new(chars),
        }
    }

    fn peek_is(&mut self, expected_char: char) -> bool {
        self.char_stream.peek_is(expected_char)
    }

    fn bump(&mut self) -> Option<char> {
        self.char_stream.bump()
    }
}

impl Scanner for CSubScanner<'_> {
    fn scan_next_word(&mut self) -> Option<Word> {
        let category = match self.bump() {
            Some('+') => Category::Plus,
            Some('-') => Category::Minus,
            Some('*') => Category::Star,
            Some('/') => Category::Slash,
            Some('<') => {
                if self.peek_is('=') {
                    self.bump();
                    Category::LessEqual
                } else {
                    Category::Less
                }
            }
            _ => return None,
        };

        let lexeme = Span {
            start: Pos::from_usize(0),
            end: self.char_stream.cur_peek_pos,
        };

        Some(Word { category, lexeme })
    }
}

#[cfg(test)]
mod tests {
    use super::{CSubScanner, Category, CharBumper, Scanner};
    use crate::source_map::{Pos, Span};

    #[test]
    fn peek_empty_input() {
        let mut bumper = CharBumper::new("".chars());
        assert_eq!(bumper.peek(), None);
    }

    #[test]
    fn peek_something() {
        let mut bumper = CharBumper::new("abc".chars());
        assert_eq!(bumper.peek(), Some('a'));
    }

    #[test]
    fn bumping_something_advances_peek() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.bump(), Some('a'));
        assert_eq!(bumper.peek(), Some('b'));

        assert_eq!(bumper.bump(), Some('b'));
        assert_eq!(bumper.peek(), Some('c'));

        assert_eq!(bumper.bump(), Some('c'));
        assert_eq!(bumper.peek(), None);

        assert_eq!(bumper.bump(), None);
        assert_eq!(bumper.peek(), None);
    }

    #[test]
    fn peeking_doesnt_change_cur_peek_position() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.cur_peek_pos, Pos::from_usize(0));

        bumper.peek();

        assert_eq!(bumper.cur_peek_pos, Pos::from_usize(0));
    }

    #[test]
    fn bumping_changes_cur_peek_position() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.cur_peek_pos, Pos::from_usize(0));

        let previous_char = bumper.bump().unwrap();

        assert_eq!(
            bumper.cur_peek_pos,
            Pos::from_usize(previous_char.len_utf8())
        );
    }

    #[test]
    fn scan_plus_token() {
        let mut scanner = CSubScanner::with_chars("+".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::Plus);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 1));
    }

    #[test]
    fn scan_minus_token() {
        let mut scanner = CSubScanner::with_chars("-".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::Minus);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 1));
    }

    #[test]
    fn scan_star_token() {
        let mut scanner = CSubScanner::with_chars("*".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::Star);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 1));
    }

    #[test]
    fn scan_slash_token() {
        let mut scanner = CSubScanner::with_chars("/".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::Slash);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 1));
    }

    #[test]
    fn scan_less_token() {
        let mut scanner = CSubScanner::with_chars("<".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::Less);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 1));
    }

    #[test]
    fn scan_less_equal_token() {
        let mut scanner = CSubScanner::with_chars("<=".chars());

        let plus_word = scanner.scan_next_word().unwrap();

        assert_eq!(plus_word.category, Category::LessEqual);
        assert_eq!(plus_word.lexeme, Span::with_usizes(0, 2));
    }
}
