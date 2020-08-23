#![allow(dead_code)]

use crate::errors::{Diag, DiagBag};
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
    Semicolon,
    Comma,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    OpenBracket,
    CloseBracket,
    Ident,
    Number,
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

#[derive(Debug, PartialEq)]
pub struct Word {
    pub category: Category,
    pub lexeme: Span,
}

impl Word {
    fn end_of_input() -> Word {
        Word {
            category: Category::Eof,
            lexeme: Span::DUMMY,
        }
    }
}

struct CharBumper<'chars> {
    char_stream: Peekable<Chars<'chars>>,
    current_peek_pos: BytePos,
}

impl<'chars> CharBumper<'chars> {
    fn new(chars: Chars<'chars>) -> CharBumper<'chars> {
        CharBumper {
            char_stream: chars.peekable(),
            current_peek_pos: BytePos(0),
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
            self.current_peek_pos = self.current_peek_pos + byte_length_in_utf8;
            Some(c)
        })
    }

    fn bump_if(&mut self, ch: char) -> bool {
        if self.peek_is(ch) {
            self.bump();
            return true;
        } else {
            return false;
        }
    }
}

type ScanResult = Result<ScanState, Diag>;

enum ScanState {
    Skipped,
    FoundCategory(Category),
    ReachedEndOfInput,
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

    fn peek(&mut self) -> Option<char> {
        self.char_stream.peek()
    }

    fn peek_is(&mut self, expected_char: char) -> bool {
        self.char_stream.peek_is(expected_char)
    }

    fn bump(&mut self) -> Option<char> {
        self.char_stream.bump()
    }

    fn bump_if(&mut self, expected_char: char) -> bool {
        self.char_stream.bump_if(expected_char)
    }

    fn scan_next_word(&mut self) -> Result<Word, DiagBag> {
        let lexeme_start = self.char_stream.current_peek_pos;
        let scan_state = self.analyse_category_and_bump_chars();
        match scan_state {
            Ok(ScanState::FoundCategory(category)) => {
                let lexeme = Span {
                    start: lexeme_start,
                    end: self.char_stream.current_peek_pos,
                };

                Ok(Word { category, lexeme })
            }
            Ok(ScanState::Skipped) => self.scan_next_word(),
            Ok(ScanState::ReachedEndOfInput) => Ok(Word::end_of_input()),
            Err(_) => todo!("diagnose errors!"),
        }
    }

    fn analyse_category_and_bump_chars(&mut self) -> ScanResult {
        let category = match self.bump() {
            Some('+') => Category::Plus,
            Some('-') => Category::Minus,
            Some('*') => Category::Star,
            Some('/') if self.bump_if('*') => {
                self.skip_block_comment();
                return Ok(ScanState::Skipped);
            }
            Some('/') => Category::Slash,
            Some('<') if self.bump_if('=') => Category::LessEqual,
            Some('<') => Category::Less,
            Some('>') if self.bump_if('=') => Category::GreaterEqual,
            Some('>') => Category::Greater,
            Some('=') if self.bump_if('=') => Category::EqualEqual,
            Some('=') => Category::Equal,
            Some('!') if self.bump_if('=') => Category::ExclamaEqual,
            Some(';') => Category::Semicolon,
            Some(',') => Category::Comma,
            Some('(') => Category::OpenParen,
            Some(')') => Category::CloseParen,
            Some('[') => Category::OpenCurly,
            Some(']') => Category::CloseCurly,
            Some('{') => Category::OpenBracket,
            Some('}') => Category::CloseBracket,
            Some('a'..='z' | 'A'..='Z') => {
                self.bump_ident();
                Category::Ident
            }
            Some('\x20' | '\n' | '\t') => return Ok(ScanState::Skipped),
            None => return Ok(ScanState::ReachedEndOfInput),
            _ => todo!("Not tested"),
        };

        Ok(ScanState::FoundCategory(category))
    }

    fn skip_block_comment(&mut self) {
        loop {
            match self.bump() {
                Some('*') if self.peek_is('/') => {
                    self.bump();
                    break;
                }
                None => todo!("diagnose missing end of block-comment!"),
                _ => {}
            }
        }
    }

    fn bump_ident(&mut self) {
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9') = self.peek() {
            self.bump();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CSubScanner, Category, CharBumper};
    use crate::{
        scanner::Word,
        source_map::{Pos, Span},
    };

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
    fn peeking_doesnt_change_current_peek_position() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.current_peek_pos, Pos::from_usize(0));

        bumper.peek();

        assert_eq!(bumper.current_peek_pos, Pos::from_usize(0));
    }

    #[test]
    fn bumping_changes_current_peek_position() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.current_peek_pos, Pos::from_usize(0));

        let previous_char = bumper.bump().unwrap();

        assert_eq!(
            bumper.current_peek_pos,
            Pos::from_usize(previous_char.len_utf8())
        );
    }

    #[test]
    fn bumps_only_if_peek_is_expected_char() {
        let mut bumper = CharBumper::new("abc".chars());

        assert_eq!(bumper.current_peek_pos, Pos::from_usize(0));
        assert!(bumper.peek_is('a'));

        // Bumps if expected char equals peek.
        assert_eq!(bumper.bump_if('a'), true);
        assert_eq!(bumper.current_peek_pos, Pos::from_usize(1));
        assert!(bumper.peek_is('b'));

        // Doesn't bump if expected char doesn't equal peek.
        assert_eq!(bumper.bump_if('!'), false);
        assert_eq!(bumper.current_peek_pos, Pos::from_usize(1));
        assert!(bumper.peek_is('b'));
    }

    fn assert_symbol(input: &str, category: Category, length: usize) {
        let mut scanner = CSubScanner::with_chars(input.chars());

        let word = scanner.scan_next_word().unwrap();

        assert_eq!(word.category, category);
        assert_eq!(word.lexeme, Span::with_usizes(0, length));
    }

    #[test]
    fn scan_next_word_advances_span_start() {
        let mut scanner = CSubScanner::with_chars("+-".chars());

        let first_word = scanner.scan_next_word().unwrap();
        assert_eq!(first_word.lexeme, Span::with_usizes(0, 1));

        let second_word = scanner.scan_next_word().unwrap();
        assert_eq!(second_word.lexeme, Span::with_usizes(1, 2));
    }

    #[test]
    fn scan_plus_token() {
        assert_symbol("+", Category::Plus, 1);
    }

    #[test]
    fn scan_minus_token() {
        assert_symbol("-", Category::Minus, 1);
    }

    #[test]
    fn scan_star_token() {
        assert_symbol("*", Category::Star, 1);
    }

    #[test]
    fn scan_slash_token() {
        assert_symbol("/", Category::Slash, 1);
    }

    #[test]
    fn scan_less_token() {
        assert_symbol("<", Category::Less, 1);
    }

    #[test]
    fn scan_less_equal_token() {
        assert_symbol("<=", Category::LessEqual, 2);
    }

    #[test]
    fn scan_greater_token() {
        assert_symbol(">", Category::Greater, 1);
    }

    #[test]
    fn scan_greater_equal_token() {
        assert_symbol(">=", Category::GreaterEqual, 2);
    }

    #[test]
    fn scan_equal_equal_token() {
        assert_symbol("==", Category::EqualEqual, 2);
    }

    #[test]
    fn scan_exclama_equal_token() {
        assert_symbol("!=", Category::ExclamaEqual, 2);
    }

    #[test]
    fn scan_equal_token() {
        assert_symbol("=", Category::Equal, 1);
    }

    #[test]
    fn scan_semicolon_token() {
        assert_symbol(";", Category::Semicolon, 1);
    }

    #[test]
    fn scan_comma_token() {
        assert_symbol(",", Category::Comma, 1);
    }

    #[test]
    fn scan_open_paren_token() {
        assert_symbol("(", Category::OpenParen, 1);
    }

    #[test]
    fn scan_close_paren_token() {
        assert_symbol(")", Category::CloseParen, 1);
    }

    #[test]
    fn scan_open_curly_token() {
        assert_symbol("[", Category::OpenCurly, 1);
    }

    #[test]
    fn scan_close_curly_token() {
        assert_symbol("]", Category::CloseCurly, 1);
    }

    #[test]
    fn scan_open_bracket_token() {
        assert_symbol("{", Category::OpenBracket, 1);
    }

    #[test]
    fn scan_close_bracket_token() {
        assert_symbol("}", Category::CloseBracket, 1);
    }

    #[test]
    fn scan_ident_head_token() {
        for letter in 'a'..='z' {
            assert_symbol(&letter.to_string(), Category::Ident, 1);
        }

        for letter in 'A'..='Z' {
            assert_symbol(&letter.to_string(), Category::Ident, 1);
        }
    }

    #[test]
    fn scan_ident_letters_and_digits_mixed_token() {
        let input_string = "H3ll0W0r1d";
        assert_symbol(&input_string, Category::Ident, input_string.len());
    }

    #[test]
    fn scan_ident_head_and_body_token() {
        let id_with_all_letters_and_digits = ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .collect::<String>();

        assert_symbol(
            &id_with_all_letters_and_digits,
            Category::Ident,
            id_with_all_letters_and_digits.len(),
        );
    }

    #[test]
    fn stop_scanning_ident_after_finding_char_other_than_letter_or_digit() {
        let ascii_start = 0u8;
        let ascii_end = 127u8;
        let chars_that_stop_ident_scanning =
            (ascii_start..=ascii_end).filter(|i| {
                (b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .chain(b'0'..=b'9')
                    .find(|ch| ch == i)
                    .is_none()
            });

        for char_that_stops_ident_scanning in chars_that_stop_ident_scanning {
            let input_string =
                format!("hello{}", char_that_stops_ident_scanning as char);

            let mut scanner = CSubScanner::with_chars(input_string.chars());

            let ident_word = scanner.scan_next_word().unwrap();
            assert_eq!(
                ident_word,
                Word {
                    category: Category::Ident,
                    lexeme: Span::with_usizes(0, 5)
                },
                "char that should stop ident scanning is {0:#X}",
                char_that_stops_ident_scanning,
            );
        }
    }

    #[test]
    fn skip_whitespace_chars() {
        let space = '\x20';
        let newline = '\x0A';
        let tab = '\x09';
        let whitespaces = &[space, newline, tab].iter().collect::<String>();

        let mut scanner = CSubScanner::with_chars(whitespaces.chars());

        let eof_word = scanner.scan_next_word().unwrap();
        assert_eq!(eof_word, Word::end_of_input());
    }

    #[test]
    fn scan_comment_block() {
        let mut scanner = CSubScanner::with_chars("/**/".chars());
        let next_word = scanner.scan_next_word().unwrap();
        assert_eq!(next_word, Word::end_of_input());
    }

    #[test]
    fn skip_everything_inside_comment_blocks() {
        let mut scanner = CSubScanner::with_chars(
            "/* this is a ++comment++!\nwith new lines!\n */".chars(),
        );

        let next_word = scanner.scan_next_word().unwrap();

        assert_eq!(next_word, Word::end_of_input());
    }

    #[test]
    fn dont_nest_comment_blocks() {
        let mut scanner = CSubScanner::with_chars("/*+/*-*/=*/".chars());

        let equal_word = scanner.scan_next_word().unwrap();
        assert_eq!(equal_word.category, Category::Equal);

        let star_word = scanner.scan_next_word().unwrap();
        assert_eq!(star_word.category, Category::Star);

        let slash_word = scanner.scan_next_word().unwrap();
        assert_eq!(slash_word.category, Category::Slash);
    }

    #[test]
    #[should_panic]
    fn missing_end_of_block_comment() {
        let mut scanner = CSubScanner::with_chars("/*".chars());
        let _ = scanner.scan_next_word();
    }
}
