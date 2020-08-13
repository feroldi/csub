use crate::source_map::BytePos;

/// A `Diag` value gathers enough information about some error in the parsing
/// process. It is used by the diagnostics system to report good quality error
/// messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Diag {
    /// Unknown character in the source code.
    UnknownCharacter { pos: BytePos },
}

#[derive(Debug)]
pub(crate) struct DiagBag {
    diags: Vec<Diag>,
}

impl DiagBag {
    fn new() -> DiagBag {
        DiagBag { diags: Vec::new() }
    }

    fn push(&mut self, diag: Diag) {
        self.diags.push(diag)
    }

    fn extend(&mut self, diag_bag: DiagBag) {
        self.diags.extend(diag_bag.diags);
    }
}

#[cfg(test)]
mod tests {
    use super::{Diag, DiagBag};
    use crate::errors::BytePos;

    #[test]
    fn new_diag_bag_is_empty() {
        let bag = DiagBag::new();
        assert!(bag.diags.is_empty());
    }

    #[test]
    fn push_to_diag_bag_adds_a_diag_to_the_end_of_diags() {
        let mut bag = DiagBag::new();
        assert!(bag.diags.is_empty());

        let diag1 = Diag::UnknownCharacter { pos: BytePos(0) };
        let diag2 = Diag::UnknownCharacter { pos: BytePos(1) };

        bag.push(diag1);
        bag.push(diag2);

        assert!(!bag.diags.is_empty());
        assert_eq!(bag.diags, vec![diag1, diag2]);
    }

    #[test]
    fn extending_from_diag_bag() {
        let diag1 = Diag::UnknownCharacter { pos: BytePos(0) };
        let diag2 = Diag::UnknownCharacter { pos: BytePos(1) };
        let diag3 = Diag::UnknownCharacter { pos: BytePos(2) };
        let diag4 = Diag::UnknownCharacter { pos: BytePos(3) };

        let mut bag1 = DiagBag {
            diags: vec![diag1, diag2],
        };

        let bag2 = DiagBag {
            diags: vec![diag3, diag4],
        };

        assert_eq!(bag1.diags, vec![diag1, diag2]);
        assert_eq!(bag2.diags, vec![diag3, diag4]);

        bag1.extend(bag2);

        assert_eq!(bag1.diags, vec![diag1, diag2, diag3, diag4]);
    }
}
