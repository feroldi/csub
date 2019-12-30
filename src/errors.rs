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

/// Forwards error diagnostics to an emitter.
struct Handler {
    emitter: Box<dyn Fn(Diag) -> bool>,
}

impl Handler {
    /// Creates a `Handler` with a custom emitter.
    pub fn with_emitter<E>(emitter: E) -> Handler
    where
        E: Fn(Diag) -> bool + 'static,
    {
        Handler {
            emitter: Box::new(emitter),
        }
    }

    /// Creates a `Handler` with an emitter that ignores all error diagnostics
    /// and returns `true`.
    pub fn with_ignoring_emitter() -> Handler {
        Handler {
            emitter: Box::new(|_| true),
        }
    }

    pub fn report(&self, diag: Diag) -> bool {
        (self.emitter)(diag)
    }
}
