use crate::source_map::BytePos;

/// A `Diag` value gathers enough information about some error in the parsing
/// process. It is used by the diagnostics system to report good quality error
/// messages.
pub(crate) enum Diag {
    /// Unknown character in the source code.
    UnknownCharacter { pos: BytePos },
    InvalidDigit { pos: BytePos },
    MissingCommentTerminator,
}
