#![allow(dead_code)]

use std::{iter::Peekable, str::Chars};

use crate::source_map::{BytePos, Pos, Span};

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

struct CSubsetScanner<'chars> {
    char_stream: Peekable<Chars<'chars>>,
    cur_peek_pos: BytePos,
}

impl<'chars> CSubsetScanner<'chars> {
    fn new(chars: Chars<'chars>) -> CSubsetScanner<'chars> {
        CSubsetScanner {
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

#[cfg(test)]
mod tests {
    use super::CSubsetScanner;
    use crate::source_map::Pos;

    #[test]
    fn peek_empty_input() {
        let mut scanner = CSubsetScanner::new("".chars());
        assert_eq!(scanner.peek(), None);
    }

    #[test]
    fn peek_something() {
        let mut scanner = CSubsetScanner::new("abc".chars());
        assert_eq!(scanner.peek(), Some('a'));
    }

    #[test]
    fn bumping_something_advances_peek() {
        let mut scanner = CSubsetScanner::new("abc".chars());

        assert_eq!(scanner.bump(), Some('a'));
        assert_eq!(scanner.peek(), Some('b'));

        assert_eq!(scanner.bump(), Some('b'));
        assert_eq!(scanner.peek(), Some('c'));

        assert_eq!(scanner.bump(), Some('c'));
        assert_eq!(scanner.peek(), None);

        assert_eq!(scanner.bump(), None);
        assert_eq!(scanner.peek(), None);
    }

    #[test]
    fn peeking_doesnt_change_cur_peek_position() {
        let mut scanner = CSubsetScanner::new("abc".chars());

        assert_eq!(scanner.cur_peek_pos, Pos::from_usize(0));

        scanner.peek();

        assert_eq!(scanner.cur_peek_pos, Pos::from_usize(0));
    }

    #[test]
    fn bumping_changes_cur_peek_position() {
        let mut scanner = CSubsetScanner::new("abc".chars());

        assert_eq!(scanner.cur_peek_pos, Pos::from_usize(0));

        let previous_char = scanner.bump().unwrap();

        assert_eq!(
            scanner.cur_peek_pos,
            Pos::from_usize(previous_char.len_utf8())
        );
    }
}
