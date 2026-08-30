const UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{};:,.<>?";

/// Look-alike characters dropped by `--no-ambiguous`.
pub const AMBIGUOUS: &str = "0O1lI";

/// The four character classes, in canonical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassKind {
    Upper,
    Lower,
    Digits,
    Symbols,
}

impl ClassKind {
    /// Declaration order matches the discriminant values, so
    /// `ALL[kind as usize]` is always `kind` itself.
    pub(crate) const ALL: [ClassKind; 4] = [
        ClassKind::Upper,
        ClassKind::Lower,
        ClassKind::Digits,
        ClassKind::Symbols,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            ClassKind::Upper => "upper",
            ClassKind::Lower => "lower",
            ClassKind::Digits => "digits",
            ClassKind::Symbols => "symbols",
        }
    }

    fn set(self) -> &'static str {
        match self {
            ClassKind::Upper => UPPER,
            ClassKind::Lower => LOWER,
            ClassKind::Digits => DIGITS,
            ClassKind::Symbols => SYMBOLS,
        }
    }

    /// This class's characters with excluded ones removed.
    pub(crate) fn pool(self, exclude: &str) -> Vec<char> {
        self.set().chars().filter(|c| !exclude.contains(*c)).collect()
    }
}

/// Which character classes are enabled for generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterClasses {
    pub upper: bool,
    pub lower: bool,
    pub digits: bool,
    pub symbols: bool,
}

impl Default for CharacterClasses {
    fn default() -> Self {
        Self {
            upper: true,
            lower: true,
            digits: true,
            symbols: true,
        }
    }
}

impl CharacterClasses {
    fn is_enabled(&self, kind: ClassKind) -> bool {
        match kind {
            ClassKind::Upper => self.upper,
            ClassKind::Lower => self.lower,
            ClassKind::Digits => self.digits,
            ClassKind::Symbols => self.symbols,
        }
    }
}

/// Returns one `(name, pool)` pair per *enabled* class, with excluded
/// characters already removed. Pools are kept separate (instead of one
/// merged charset) so `require_each` can check membership per class.
pub(crate) fn class_pools(
    classes: &CharacterClasses,
    exclude: &str,
) -> Vec<(&'static str, Vec<char>)> {
    ClassKind::ALL
        .into_iter()
        .filter(|kind| classes.is_enabled(*kind))
        .map(|kind| (kind.name(), kind.pool(exclude)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_classes_are_omitted() {
        let classes = CharacterClasses {
            upper: true,
            lower: false,
            digits: true,
            symbols: false,
        };
        let pools = class_pools(&classes, "");
        let names: Vec<&str> = pools.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, ["upper", "digits"]);
    }

    #[test]
    fn excluded_characters_are_removed_from_their_pool() {
        let pools = class_pools(&CharacterClasses::default(), "ABC012");
        let upper = &pools[0].1;
        let digits = &pools[2].1;
        assert_eq!(upper.len(), 23);
        assert_eq!(digits.len(), 7);
        assert!(!upper.contains(&'A'));
        assert!(!digits.contains(&'0'));
    }

    #[test]
    fn ambiguous_set_only_contains_look_alikes() {
        let pools = class_pools(&CharacterClasses::default(), AMBIGUOUS);
        for (_, pool) in pools {
            assert!(pool.iter().all(|c| !AMBIGUOUS.contains(*c)));
        }
    }

    #[test]
    fn all_order_matches_discriminants() {
        for (index, kind) in ClassKind::ALL.into_iter().enumerate() {
            assert_eq!(kind as usize, index);
        }
    }
}
