use std::{
    ops::{Add, Sub},
    rc::Rc,
};

/// A byte position or offset into a source file's text buffer. This is used to
/// map ASTs to soure code by indicating the position from which an AST node
/// was parsed.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BytePos(pub usize);

/// A range (span) into a source file's text buffer, indicating a region of
/// text.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

impl Span {
    pub(crate) fn with_usizes(start: usize, end: usize) -> Span {
        Span {
            start: Pos::from_usize(start),
            end: Pos::from_usize(end),
        }
    }
}

/// Conversion from offsets (e.g. `BytePos`) to arithmetic values and
/// vice-versa.
pub trait Pos {
    fn from_usize(value: usize) -> Self;
    fn to_usize(&self) -> usize;
}

pub const DUMMY_BYTEPOS: BytePos = BytePos(0);
pub const DUMMY_SPAN: Span = Span {
    start: DUMMY_BYTEPOS,
    end: DUMMY_BYTEPOS,
};

impl Pos for BytePos {
    fn from_usize(value: usize) -> BytePos {
        BytePos(value)
    }

    fn to_usize(&self) -> usize {
        self.0
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 + rhs.0)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 - rhs.0)
    }
}

/// A source location containing line and column number. Useful for diagnostics.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Loc {
    pub line: usize,
    pub col: BytePos,
}

/// This holds information of a given source file, such as the source name,
/// text buffer, line positions etc.
///
/// A `SourceFile` assists in reporting errors and mapping ASTs to source code,
/// providing an interface for text information lookup, such as: line and
/// column number for a given position; text snippets from spans etc.
pub struct SourceFile {
    /// File's content.
    pub src: Rc<String>,
    /// Byte positions following every new line.
    start_of_line_positions: Vec<BytePos>,
}

impl SourceFile {
    /// Constructs a new `SourceFile` from a string (the text buffer).
    ///
    /// Line positions are precomputed by this function.
    pub fn new(source_content: String) -> SourceFile {
        let mut start_of_line_positions = vec![BytePos(0)];

        for (i, b) in source_content.bytes().enumerate() {
            if b == b'\n' {
                start_of_line_positions.push(BytePos(i + 1));
            }
        }

        start_of_line_positions.push(BytePos(source_content.len()));

        SourceFile {
            src: Rc::new(source_content),
            start_of_line_positions,
        }
    }

    /// Returns a string slice represented by a `Span`.
    pub fn span_to_snippet(&self, s: Span) -> &str {
        &self.src[s.start.0..s.end.0]
    }

    /// Returns the line number for a `BytePos` if such is valid.
    pub fn lookup_line_index(&self, pos: BytePos) -> Option<usize> {
        let pos_index = pos.to_usize();
        for (i, line_pos) in self.start_of_line_positions.iter().enumerate() {
            let line_pos_index = line_pos.to_usize();
            if pos_index < line_pos_index {
                return Some(i - 1);
            }
        }

        None
    }

    /// Returns the source information (line/column number etc) of a
    /// `BytePos` if such is valid.
    pub fn lookup_source_location(&self, pos: BytePos) -> Option<Loc> {
        self.lookup_line_index(pos).map(|line_index| {
            let line = line_index + 1;
            let col = pos - self.start_of_line_positions[line_index];

            Loc { line, col }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BytePos, Loc, Pos, SourceFile, Span, DUMMY_BYTEPOS, DUMMY_SPAN};

    fn create_source_file() -> SourceFile {
        SourceFile::new("first line.\nsecond line.\nthird line.\n".into())
    }

    #[test]
    fn dummy_byte_positions() {
        assert_eq!(DUMMY_BYTEPOS, BytePos(0));
        assert_eq!(
            DUMMY_SPAN,
            Span {
                start: DUMMY_BYTEPOS,
                end: DUMMY_BYTEPOS
            }
        );
    }

    #[test]
    fn calc_line_positions_test() {
        let source_file = create_source_file();

        assert_eq!(BytePos(0), source_file.start_of_line_positions[0]);
        assert_eq!(BytePos(12), source_file.start_of_line_positions[1]);
        assert_eq!(BytePos(25), source_file.start_of_line_positions[2]);
        assert_eq!(BytePos(37), source_file.start_of_line_positions[3]);
    }

    #[test]
    fn get_snippets_from_span_test() {
        let source_file = create_source_file();

        let s = Span {
            start: BytePos(0),
            end: BytePos(5),
        };
        assert_eq!("first", source_file.span_to_snippet(s));

        let s = Span {
            start: BytePos(12),
            end: BytePos(18),
        };
        assert_eq!("second", source_file.span_to_snippet(s));
    }

    #[test]
    fn lookup_line_indicies_test() {
        let source_file = create_source_file();

        assert_eq!(Some(0), source_file.lookup_line_index(BytePos(0)));
        assert_eq!(Some(0), source_file.lookup_line_index(BytePos(1)));
        assert_eq!(Some(1), source_file.lookup_line_index(BytePos(12)));
        assert_eq!(Some(2), source_file.lookup_line_index(BytePos(25)));
        assert_eq!(None, source_file.lookup_line_index(BytePos(37)));
    }

    #[test]
    fn lookup_source_locations_test() {
        let source_file = create_source_file();

        assert_eq!(
            Some(Loc {
                line: 1,
                col: BytePos(0),
            }),
            source_file.lookup_source_location(BytePos(0))
        );

        assert_eq!(
            Some(Loc {
                line: 1,
                col: BytePos(3),
            }),
            source_file.lookup_source_location(BytePos(3))
        );

        assert_eq!(
            Some(Loc {
                line: 2,
                col: BytePos(0),
            }),
            source_file.lookup_source_location(BytePos(12))
        );

        assert_eq!(
            Some(Loc {
                line: 2,
                col: BytePos(3),
            }),
            source_file.lookup_source_location(BytePos(15))
        );

        assert_eq!(None, source_file.lookup_source_location(BytePos(37)));
    }

    #[test]
    fn span_from_usizes() {
        let span = Span::with_usizes(0, 42);

        assert_eq!(span.start, Pos::from_usize(0));
        assert_eq!(span.end, Pos::from_usize(42));
    }
}
