use std::{iter::Peekable, str::Chars};

use crate::source_map::{BytePos, Pos, Span};

enum Category {
    Kw(Keyword),
    Exclama,
    AmpAmp,
    PipePipe,
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

struct Scanner<'chars> {
    char_stream: Peekable<Chars<'chars>>,
    cur_bpos: BytePos,
}

impl Scanner<'_> {
    fn peek(&mut self) -> Option<char> {
        self.char_stream.peek().map(|&c| c)
    }

    fn consume(&mut self) -> Option<char> {
        self.char_stream.next().and_then(|c| {
            self.cur_bpos = self.cur_bpos + Pos::from_usize(c.len_utf8());
            Some(c)
        })
    }
}
