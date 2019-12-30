use std::{iter::Peekable, rc::Rc, str::Chars};

use crate::{
    errors::Diag,
    source_map::{BytePos, Pos, SourceFile, Span, DUMMY_SPAN},
};

enum Category {
    Kw(Keyword),
    // Exclama,
    // AmpAmp,
    // PipePipe,
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

enum Keyword {
    Else,
    If,
    Int,
    Return,
    Void,
    While,
}

struct Word {
    category: Category,
    lexeme: Span,
}

impl Word {
    fn eof() -> Word {
        Word {
            category: Category::Eof,
            lexeme: DUMMY_SPAN,
        }
    }
}

struct Scanner<'chars> {
    char_stream: Peekable<Chars<'chars>>,
    source_file: Rc<SourceFile>,
    cur_bpos: BytePos,
}

impl Scanner<'_> {
    // FIXME(feroldi): Scanner::new should construct a peekable stream of chars from
    // the source file, instead of being passed as a function argument.
    fn new<'chars>(
        char_stream: Peekable<Chars<'chars>>,
        source_file: Rc<SourceFile>,
    ) -> Scanner<'chars> {
        Scanner {
            char_stream,
            source_file,
            cur_bpos: Pos::from_usize(0),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.char_stream.peek().map(|&c| c)
    }

    fn peek_is(&mut self, ch: char) -> bool {
        self.peek() == Some(ch)
    }

    fn bump(&mut self) -> Option<char> {
        self.char_stream.next().and_then(|c| {
            self.cur_bpos = self.cur_bpos + Pos::from_usize(c.len_utf8());
            Some(c)
        })
    }

    fn next_word(&mut self) -> Result<Word, Diag> {
        while is_whitespace(self.peek()) {
            self.bump();
        }

        let word_start_bpos = self.cur_bpos;

        let category = match self.bump() {
            Some('+') => Category::Plus,
            Some('-') => Category::Minus,
            Some('*') => Category::Star,
            Some('/') => {
                if self.peek_is('*') {
                    self.bump();
                    loop {
                        if self.peek_is('*') {
                            self.bump();
                            if self.peek_is('/') {
                                self.bump();
                                return self.next_word();
                            }
                        }
                        match self.bump() {
                            Some('\0') | None => return Err(Diag::MissingCommentTerminator),
                            _ => {}
                        }
                    }
                } else {
                    Category::Slash
                }
            }
            Some(';') => Category::Semi,
            Some(',') => Category::Comma,
            Some('(') => Category::OpenParen,
            Some(')') => Category::CloseParen,
            Some('{') => Category::OpenCurly,
            Some('}') => Category::CloseCurly,
            Some('[') => Category::OpenBracket,
            Some(']') => Category::CloseBracket,
            Some('<') => {
                if self.peek_is('=') {
                    self.bump();
                    Category::LessEqual
                } else {
                    Category::Less
                }
            }
            Some('>') => {
                if self.peek_is('=') {
                    self.bump();
                    Category::GreaterEqual
                } else {
                    Category::Greater
                }
            }
            Some('=') => {
                if self.peek_is('=') {
                    self.bump();
                    Category::EqualEqual
                } else {
                    Category::Equal
                }
            }
            Some('!') => {
                if self.peek_is('=') {
                    self.bump();
                    Category::ExclamaEqual
                } else {
                    return Err(Diag::UnknownCharacter {
                        pos: word_start_bpos,
                    });
                }
            }
            ch if is_letter(ch) => return self.scan_ident(word_start_bpos),
            ch if is_digit(ch) => return self.scan_number(word_start_bpos),
            Some('\0') | None => Category::Eof,
            Some(_) => {
                return Err(Diag::UnknownCharacter {
                    pos: word_start_bpos,
                })
            }
        };

        Ok(Word {
            category,
            lexeme: Span {
                start: word_start_bpos,
                end: self.cur_bpos,
            },
        })
    }

    fn scan_ident(&mut self, ident_start_pos: BytePos) -> Result<Word, Diag> {
        assert!(is_alphanum(self.peek()));

        while is_alphanum(self.peek()) {
            self.bump();
        }

        let lexeme = Span {
            start: ident_start_pos,
            end: self.cur_bpos,
        };

        let category = match self.source_file.span_to_snippet(lexeme) {
            "else" => Category::Kw(Keyword::Else),
            "if" => Category::Kw(Keyword::If),
            "int" => Category::Kw(Keyword::Int),
            "return" => Category::Kw(Keyword::Return),
            "void" => Category::Kw(Keyword::Void),
            "while" => Category::Kw(Keyword::While),
            _ => Category::Ident,
        };

        Ok(Word { category, lexeme })
    }

    fn scan_number(&mut self, number_start_pos: BytePos) -> Result<Word, Diag> {
        assert!(is_digit(self.peek()));

        while is_digit(self.peek()) {
            self.bump();
        }

        if is_letter(self.peek()) {
            return Err(Diag::InvalidDigit { pos: self.cur_bpos });
        }

        Ok(Word {
            category: Category::Number,
            lexeme: Span {
                start: number_start_pos,
                end: self.cur_bpos,
            },
        })
    }
}

fn is_letter(ch: Option<char>) -> bool {
    match ch {
        Some('a'..='z') | Some('A'..='Z') => true,
        _ => false,
    }
}

fn is_digit(ch: Option<char>) -> bool {
    match ch {
        Some('0'..='9') => true,
        _ => false,
    }
}

fn is_alphanum(ch: Option<char>) -> bool {
    is_letter(ch) || is_digit(ch)
}

fn is_whitespace(ch: Option<char>) -> bool {
    match ch {
        Some(' ') | Some('\n') | Some('\t') => true,
        _ => false,
    }
}
